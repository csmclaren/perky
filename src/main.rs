use core::{array, cmp, error::Error, iter, ops::RangeInclusive, time::Duration, u64};

use std::{
    collections::{BTreeMap, HashSet},
    env,
    io::Write,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::Instant,
};

use clap::{ArgAction, Parser, ValueEnum};

use rayon::ThreadPoolBuilder;

use serde_json::Value;

use termcolor::BufferedStandardStream;

use perky::{
    expressions::Expression,
    goals,
    json::write_json_flatten_primitive_arrays,
    keys::{Key, KeyTable},
    layouts::LayoutTable,
    measurements::Measurement,
    metadata::Metadata,
    metrics::{self, partition_sort_rules},
    ngrams::{
        read_bigram_table_from_bytes, read_bigram_table_from_path, read_trigram_table_from_bytes,
        read_trigram_table_from_path, read_unigram_table_from_bytes, read_unigram_table_from_path,
        sum_ngram_table,
    },
    permutations::{convert_option_vec_to_array, permute_and_substitute},
    records::{Record, filter_records, select_records, sort_records},
    scores::{
        ScoreMode, score_bfs, score_bfs_without_details_unsafe, score_tfs,
        score_tfs_without_details_unsafe, score_ufs, score_ufs_without_details_unsafe,
    },
    ui::{self, styles::WriteStyled},
    util::{math::factorial, signals::ignore_sigpipe, strings::unescape, threads::throttle},
    weights,
    writers::{write_progress, write_records_json, write_records_text},
};

const C: usize = 16;
const R: usize = 8;

const PERMIT_PARTIAL_PERMUTATIONS: bool = true;

const DEFAULT_1_GRAMS: &[u8] = include_bytes!("../resources/charfreq-google/1-grams-uc.tsv");
const DEFAULT_2_GRAMS: &[u8] = include_bytes!("../resources/charfreq-google/2-grams-uc.tsv");
const DEFAULT_3_GRAMS: &[u8] = include_bytes!("../resources/charfreq-google/3-grams-uc.tsv");

// Cli

#[derive(Parser)]
#[command(about, author, long_about = None, version)]
struct Cli {
    /// Path to layout table file. [default: 'default.lt.json']
    ///
    /// This must be a valid JSON file in the layout table format.
    #[arg(short, long = "layout-table", value_name = "FPATH")]
    layout_table_fpath: Option<PathBuf>,

    /// Path to key table file. [default: 'default.kt.json']
    ///
    /// This must be a valid JSON file in the key table format
    #[arg(short, long = "key-table", value_name = "FPATH")]
    key_table_fpath: Option<PathBuf>,

    /// Path to unigram table file.
    ///
    /// This must be a valid TSV file.
    /// Each line must have a unigram in column 0 and count in column 1.
    #[arg(short, long = "unigram-table", value_name = "FPATH")]
    unigram_table_fpath: Option<PathBuf>,

    /// Path to bigram table file.
    ///
    /// This must be a valid TSV file.
    /// Each line must have a bigram in column 0 and count in column 1.
    #[arg(short, long = "bigram-table", value_name = "FPATH")]
    bigram_table_fpath: Option<PathBuf>,

    /// Path to trigram table file.
    ///
    /// This must be a valid TSV file.
    /// Each line must have a trigram in column 0 and count in column 1.
    #[arg(short, long = "trigram-table", value_name = "FPATH")]
    trigram_table_fpath: Option<PathBuf>,

    /// Goal for the selected metric.
    ///
    /// This overrides the default goal for the metric.
    #[arg(short = 'g', long, value_name = "GOAL")]
    goal: Option<Goal>,

    /// Metric used for scoring.
    ///
    /// This metric will be used for evaluating key tables.
    #[arg(
        short = 'm',
        long,
        default_value = "sfb",
        value_enum,
        value_name = "METRIC"
    )]
    metric: Metric,

    /// Tolerance for the selected metric.
    ///
    /// Results within this tolerance of the best score will be retained.
    /// Permitted range is 0.0 to 1.0.
    #[arg(long,
        default_value_t = 1.0,
        hide = true,
        value_name = "TOLERANCE",
        value_parser = validate_tolerance
    )]
    tolerance: f64,

    /// Weighing method used for the selected metric.
    #[arg(short = 'w', long, value_name = "WEIGHT")]
    weight: Option<Weight>,

    /// Characters to substitute for any '1's in key table.
    ///
    /// Substitution order is left to right, top to bottom.
    #[arg(short = '1', long, value_name = "STRING")]
    region1: Option<String>,

    /// Characters to substitute for any '2's in key table.
    ///
    /// Substitution order is left to right, top to bottom.
    #[arg(short = '2', long, value_name = "STRING")]
    region2: Option<String>,

    /// Characters to substitute for any '3's in key table.
    ///
    /// Substitution order is left to right, top to bottom.
    #[arg(short = '3', long, value_name = "STRING")]
    region3: Option<String>,

    /// Use parallel execution algorithm.
    ///
    /// Setting this to false will force the use of a specialized
    /// single-threaded algorithm. This is not equivalent to --threads 1
    /// (which uses the parallel execution algorithm, but for a single thread).
    #[arg(long, action = ArgAction::Set, hide = true, default_value_t = true)]
    parallelize: bool,

    /// Maximum number of permutations to consider.
    #[arg(short = 'p', long)]
    permutations: Option<u64>,

    /// Number of nanoseconds to yield threads per permutation batch.
    #[arg(long, default_value_t = 0)]
    sleep_ns: u64,

    /// Number of threads to use for parallel execution.
    /// 0 means use all logical cores.
    #[arg(long, default_value_t = 0)]
    threads: usize,

    /// Maximum number of results to process before sorting, filtering, and selecting.
    ///
    /// An unreasonably large number of results can cause the post-processing steps to take a long
    /// time to complete.
    #[arg(long, value_name = "N", default_value_t = 10000)]
    truncate: u64,

    /// Metrics to sort in ascending order.
    ///
    /// May be specified multiple times, with multiple metrics each time.
    /// Can be interleaved with '--sort-desc'.
    #[arg(
        long = "sort-asc",
        action = ArgAction::Append,
        num_args = 1..,
        value_enum,
        value_name = "METRIC"
    )]
    sort_asc: Vec<Metric>,

    /// Metrics to sort in descending order.
    ///
    /// May be specified multiple times, with multiple metrics each time.
    /// Can be interleaved with '--sort-asc'.
    #[arg(
        long = "sort-desc",
        action = ArgAction::Append,
        num_args = 1..,
        value_enum,
        value_name = "METRIC"
    )]
    sort_desc: Vec<Metric>,

    /// Filter expression.
    ///
    /// May be specified multiple times.
    #[arg(
        short = 'f',
        long = "filter",
        action = ArgAction::Append,
        num_args = 1,
        value_name = "EXPRESSION"
    )]
    filters: Vec<String>,

    /// Select a specific record by index. Negative values count from the end.
    #[arg(short = 'i', long)]
    index: Option<isize>,

    /// Maximum number of records to print.
    ///
    /// This is similar to truncate, but occurs after sorting, filtering, and selecting.
    #[arg(short = 'r', long)]
    max_records: Option<usize>,

    /// Format for printing.
    #[arg(long, default_value = "text", value_enum)]
    format: Format,

    /// Print metadata.
    ///
    /// If not specified, metadata is printed only when there is more than one permutation.
    #[arg(long, action = ArgAction::Set)]
    print_metadata: Option<bool>,

    /// Show detailed information for specific metrics.
    #[arg(long, num_args = 1.., value_enum, value_name = "METRIC")]
    print_details: Vec<Metric>,

    /// Show summaries of metrics.
    #[arg(long, action = ArgAction::Set, default_value_t = true)]
    print_summaries: bool,

    /// Print percentages.
    #[arg(long, action = ArgAction::Set, default_value_t = true)]
    print_perc: bool,

    /// Specify when colours and text effects may be used.
    #[arg(long = "style", default_value_t = StylePolicy::Auto, value_enum, value_name = "STYLE")]
    style_policy: StylePolicy,
}

fn validate_tolerance(s: &str) -> Result<f64, String> {
    const RANGE: RangeInclusive<f64> = 0.0..=1.0;
    s.parse::<f64>()
        .map_err(|_| format!("value must be a floating-point number, found '{}'", s))
        .and_then(|v| {
            if RANGE.contains(&v) {
                Ok(v)
            } else {
                Err(format!(
                    "value must be a floating-point number between {} and {} (inclusive), found {}",
                    RANGE.start(),
                    RANGE.end(),
                    v
                ))
            }
        })
}

// Format

#[derive(Clone, ValueEnum)]
enum Format {
    Json,
    Text,
}

// Goal

#[derive(Clone, ValueEnum)]
enum Goal {
    /// Maximize.
    Max,
    /// Minimize.
    Min,
}

impl From<&Goal> for goals::Goal {
    fn from(value: &Goal) -> Self {
        use Goal::*;
        match value {
            Max => Self::Max,
            Min => Self::Min,
        }
    }
}

// Metric

#[derive(Clone, ValueEnum)]
enum Metric {
    // Unigram metrics
    Lt,
    Li,
    Lm,
    Lr,
    Lp,
    Lh,
    Rt,
    Ri,
    Rm,
    Rr,
    Rp,
    Rh,
    // Bigram metrics
    Fsb,
    Hsb,
    Irb,
    Lsb,
    Orb,
    Sfb,
    // Trigram metrics
    Alt,
    One,
    Red,
    Rol,
}

macro_rules! map_metrics {
    (
        $(
            $( $variant:ident ),+ => ($variant_enum:ident, $sub_enum:ident)
        ),* $(,)?
    ) => {
        impl From<&$crate::Metric> for $crate::metrics::Metric {
            fn from(value: &$crate::Metric) -> Self {
                match value {
                    $(
                        $(
                            $crate::Metric::$variant => $crate::metrics::Metric::$variant_enum(
                                $crate::metrics::$sub_enum::$variant
                            ),
                        )+
                    )*
                }
            }
        }
    }
}

map_metrics! {
    Lt, Li, Lm, Lr, Lp, Lh, Rt, Ri, Rm, Rr, Rp, Rh => (Unigram, UnigramMetric),
    Fsb, Hsb, Irb, Lsb, Orb, Sfb => (Bigram, BigramMetric),
    Alt, One, Red, Rol => (Trigram, TrigramMetric)
}

// SortRule

fn parse_sort_rules() -> Result<Vec<metrics::SortRule>, Box<dyn Error>> {
    let mut result = Vec::new();
    let mut arguments: Box<dyn Iterator<Item = String>> = Box::new(env::args().skip(1));
    while let Some(argument) = arguments.next() {
        let sort_direction = if argument == "--sort-asc" {
            metrics::SortDirection::Ascending
        } else if argument == "--sort-desc" {
            metrics::SortDirection::Descending
        } else {
            continue;
        };
        while let Some(next_argument) = arguments.next() {
            if next_argument.starts_with("--") {
                arguments = Box::new(iter::once(next_argument).chain(arguments));
                break;
            }
            let metric = metrics::Metric::from(&Metric::from_str(&next_argument, true)?);
            result.push(metrics::SortRule {
                metric,
                sort_direction: sort_direction.clone(),
            });
        }
    }
    Ok(result)
}

// StylePolicy

#[derive(Clone, ValueEnum)]
enum StylePolicy {
    /// Use colours and text effects when printing to a terminal.
    Auto,
    /// Never use colours or text effects.
    Off,
    /// Always use colours and text effects.
    On,
}

impl From<&StylePolicy> for ui::styles::StylePolicy {
    fn from(value: &StylePolicy) -> Self {
        use StylePolicy::*;
        match value {
            Auto => Self::Auto,
            Off => Self::Off,
            On => Self::On,
        }
    }
}

// Weight

#[derive(Clone, ValueEnum)]
enum Weight {
    /// Weigh by n-gram counts and effort.
    Effort,
    /// Weigh only by n-gram counts.
    Raw,
}

impl From<&Weight> for weights::Weight {
    fn from(value: &Weight) -> Self {
        use Weight::*;
        match value {
            Effort => Self::Effort,
            Raw => Self::Raw,
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    ignore_sigpipe();

    // Argument parsing

    let cli = Cli::parse();

    // Argument parsing (files)

    let layout_table_fpath = cli
        .layout_table_fpath
        .unwrap_or_else(|| PathBuf::from("default.lt.json"));

    let mut layout_table =
        LayoutTable::<C, R>::read_from_path(&layout_table_fpath).map_err(|e| {
            format!(
                "Failed to load file '{}': {e}",
                layout_table_fpath.display()
            )
        })?;

    let key_table_fpath = cli
        .key_table_fpath
        .unwrap_or_else(|| PathBuf::from("default.kt.json"));

    let key_table = KeyTable::read_from_path(&key_table_fpath)
        .map_err(|e| format!("Failed to load file '{}': {e}", key_table_fpath.display()))?;

    let unigram_table = match cli.unigram_table_fpath {
        None => read_unigram_table_from_bytes(DEFAULT_1_GRAMS)?,
        Some(fname) => {
            let fpath = Path::new(&fname);
            read_unigram_table_from_path(fpath)
                .map_err(|e| format!("Failed to load file '{}': {e}", fpath.display()))?
        }
    };

    let bigram_table = match cli.bigram_table_fpath {
        None => read_bigram_table_from_bytes(DEFAULT_2_GRAMS)?,
        Some(fname) => {
            let fpath = Path::new(&fname);
            read_bigram_table_from_path(fpath)
                .map_err(|e| format!("Failed to load file '{}': {e}", fpath.display()))?
        }
    };

    let trigram_table = match cli.trigram_table_fpath {
        None => read_trigram_table_from_bytes(DEFAULT_3_GRAMS)?,
        Some(fname) => {
            let fpath = Path::new(&fname);
            read_trigram_table_from_path(fpath)
                .map_err(|e| format!("Failed to load file '{}': {e}", fpath.display()))?
        }
    };

    // Argument parsing (scoring)

    let goal = goals::Goal::from(&cli.goal.unwrap_or(Goal::Min));

    let metric = metrics::Metric::from(&cli.metric);

    let tolerance = cli.tolerance;

    let weight = weights::Weight::from(&cli.weight.unwrap_or(Weight::Raw));

    // Argument parsing (permuting)

    let opt_vec1 = match &cli.region1 {
        None => None,
        Some(s) => {
            let s = unescape::<true>(s).map_err(|e| format!("Invalid -1 argument: {}", e))?;
            if !s.is_ascii() || s.chars().any(|ch| ('\x01'..='\x03').contains(&ch)) {
                Err(
                    "Invalid -1 argument: Characters must be ASCII, and the control characters SOH, STX, and ETX are reserved.",
                )?;
            }
            Some(s.into_bytes())
        }
    };

    let opt_vec2 = match &cli.region2 {
        None => None,
        Some(s) => {
            let s = unescape::<true>(s).map_err(|e| format!("Invalid -2 argument: {}", e))?;
            if !s.is_ascii() || s.chars().any(|ch| ('\x01'..='\x03').contains(&ch)) {
                Err(
                    "Invalid -2 argument: Characters must be ASCII, and the control characters SOH, STX, and ETX are reserved.",
                )?;
            }
            Some(s.into_bytes())
        }
    };

    let opt_vec3 = match &cli.region3 {
        None => None,
        Some(s) => {
            let s = unescape::<true>(s).map_err(|e| format!("Invalid -3 argument: {}", e))?;
            if !s.is_ascii() || s.chars().any(|ch| ('\x01'..='\x03').contains(&ch)) {
                Err(
                    "Invalid -3 argument: Characters must be ASCII, and the control characters SOH, STX, and ETX are reserved.",
                )?;
            }
            Some(s.into_bytes())
        }
    };

    let (array1, length1) = convert_option_vec_to_array::<256, _>(opt_vec1)?;
    let (array2, length2) = convert_option_vec_to_array::<256, _>(opt_vec2)?;
    let (array3, length3) = convert_option_vec_to_array::<256, _>(opt_vec3)?;

    let mut coordinates1 = Vec::new();
    let mut coordinates2 = Vec::new();
    let mut coordinates3 = Vec::new();

    for (r, row) in key_table.0.iter().enumerate() {
        for (c, cell) in row.iter().enumerate() {
            use Key::*;
            match cell {
                Some(One) => coordinates1.push((r, c)),
                Some(Two) => coordinates2.push((r, c)),
                Some(Three) => coordinates3.push((r, c)),
                _ => (),
            };
        }
    }

    let len_1s = coordinates1.len();
    if len_1s >= 1 {
        if length1 == 0 {
            if !PERMIT_PARTIAL_PERMUTATIONS {
                Err(format!(
                    "There are ({}) 1s in the key table. \
                     Provide a string for permutation of the same length via '-1'",
                    len_1s
                ))?
            }
        } else if length1 != len_1s {
            Err(format!(
                "There are ({}) 1s in the key table, \
                 but the length of '-1' is {}. \
                 Provide a string for permutation of the same length via '-1'",
                len_1s, length1
            ))?
        }
    }

    let len_2s = coordinates2.len();
    if len_2s >= 1 {
        if length2 == 0 {
            if !PERMIT_PARTIAL_PERMUTATIONS {
                Err(format!(
                    "There are ({}) 2s in the key table. \
                     Provide a string for permutation of the same length via '-2'",
                    len_2s
                ))?
            }
        } else if length2 != len_2s {
            Err(format!(
                "There are ({}) 2s in the key table, \
                 but the length of '-2' is {}. \
                 Provide a string for permutation of the same length via '-2'",
                len_2s, length2
            ))?
        }
    }

    let len_3s = coordinates3.len();
    if len_3s >= 1 {
        if length3 == 0 {
            if !PERMIT_PARTIAL_PERMUTATIONS {
                Err(format!(
                    "There are ({}) 3s in the key table. \
                     Provide a string for permutation of the same length via '-3'",
                    len_3s
                ))?
            }
        } else if length3 != len_3s {
            Err(format!(
                "There are ({}) 3s in the key table, \
                 but the length of '-3' is {}. \
                 Provide a string for permutation of the same length via '-3'",
                len_3s, length3
            ))?
        }
    }

    let parallelize = cli.parallelize;

    let permutations = cli.permutations.unwrap_or(u64::MAX);

    let sleep_ns = cli.sleep_ns;

    let threads = cli.threads;

    if threads >= 1 {
        ThreadPoolBuilder::new()
            .num_threads(threads)
            .build_global()
            .map_err(|e| format!("Failed to initialize thread pool: {}", e))?;
    }

    // Argument parsing (truncating)

    let truncate = cli.truncate;

    // Argument parsing (sorting)

    let sort_rules = parse_sort_rules()?;

    let (
        _unigram_metrics_required_for_sorting,
        _bigram_metrics_required_for_sorting,
        _trigram_metrics_required_for_sorting,
    ) = partition_sort_rules(&sort_rules);

    // Argument parsing (filtering)

    let filters = cli
        .filters
        .into_iter()
        .map(|s| Expression::parse(s.as_str(), &metrics::Metric::get_variables()))
        .collect::<Result<Vec<_>, _>>()?;

    // Argument parsing (selecting)

    let opt_max_records = cli.max_records;

    let opt_index = cli.index;

    // Argument parsing (printing)

    let format = cli.format;

    let print_metadata = cli.print_metadata;

    let print_summaries = cli.print_summaries;

    let print_details = cli
        .print_details
        .iter()
        .map(metrics::Metric::from)
        .collect::<Vec<_>>();

    let print_perc = cli.print_perc;

    let style_policy = ui::styles::StylePolicy::from(&cli.style_policy);

    // Permuting (setup)

    let stderr = BufferedStandardStream::stderr(style_policy.color_choice());
    let mut stdout = BufferedStandardStream::stdout(style_policy.color_choice());

    layout_table.mask(|r, c, _digit| key_table.0[r][c].is_some());

    let unigram_fingerings = layout_table.unigram_fingerings();
    let bigram_fingerings = layout_table.bigram_fingerings();
    let trigram_fingerings = layout_table.trigram_fingerings();

    let scoring_fn = |key_table_matrix: &[[u8; C]; R]| {
        let (score, score_ew) = match metric {
            metrics::Metric::Unigram(unigram_metric) => score_ufs_without_details_unsafe(
                unigram_fingerings.get_by_metric(unigram_metric),
                key_table_matrix,
                &unigram_table,
            ),
            metrics::Metric::Bigram(bigram_metric) => score_bfs_without_details_unsafe(
                bigram_fingerings.get_by_metric(bigram_metric),
                key_table_matrix,
                &bigram_table,
            ),
            metrics::Metric::Trigram(trigram_metric) => score_tfs_without_details_unsafe(
                trigram_fingerings.get_by_metric(trigram_metric),
                key_table_matrix,
                &trigram_table,
            ),
        };
        use weights::Weight::*;
        match weight {
            Effort => score_ew,
            Raw => score,
        }
    };

    let key_table_matrix = key_table.to_byte_matrix();

    let expected_permutations = cmp::min(
        permutations,
        factorial(length1 as u64) * factorial(length2 as u64) * factorial(length3 as u64),
    );

    // Permuting (main)

    let start = Instant::now();

    let should_write_progress = expected_permutations > 1;
    let stderr = Arc::new(Mutex::new(stderr));
    let stderr_clone = Arc::clone(&stderr);

    let progress_fn = throttle(
        move |i: u64| {
            if should_write_progress {
                let mut stderr = stderr_clone.lock().unwrap();
                write_progress(
                    &mut *stderr,
                    i,
                    Some(expected_permutations),
                    Some(start.elapsed()),
                    true,
                    1,
                    None,
                    None,
                )
                .ok();
            }
        },
        Duration::from_millis(200),
    );

    let (mut key_table_matrices, score, total_permutations) = permute_and_substitute(
        &key_table_matrix,
        (array1, length1, &coordinates1),
        (array2, length2, &coordinates2),
        (array3, length3, &coordinates3),
        progress_fn,
        scoring_fn,
        goal,
        tolerance,
        permutations,
        parallelize,
        sleep_ns,
        Some(truncate),
    )?;

    let mut stderr = stderr.lock().unwrap();

    if should_write_progress {
        writeln!(stderr)?;
        stderr.flush()?;
    }

    let elapsed_duration = start.elapsed();

    // Permuting (teardown)

    let total_records = key_table_matrices.len();

    // Deduplicating

    let mut seen = HashSet::new();
    key_table_matrices.retain(|k| seen.insert(k.clone()));
    let total_unique_records = key_table_matrices.len();

    // Measuring

    let mut records: Vec<_> = key_table_matrices
        .into_iter()
        .map(|key_table_matrix| {
            let unigram_measurements = metrics::UnigramMetric::VARIANT_ARRAY
                .iter()
                .map(|&metric| {
                    let fs = unigram_fingerings.get_by_metric(metric);
                    let score_mode = if print_details.contains(&metrics::Metric::Unigram(metric)) {
                        ScoreMode::Detailed
                    } else {
                        ScoreMode::SummaryUnsafe
                    };
                    let (opt_details, f_sum, f_sum_ew) =
                        score_ufs(fs, &key_table_matrix, &unigram_table, score_mode);
                    (metric, Measurement::new(opt_details, f_sum, f_sum_ew))
                })
                .collect::<BTreeMap<_, _>>();

            let bigram_measurements = metrics::BigramMetric::VARIANT_ARRAY
                .iter()
                .map(|&metric| {
                    let fs = bigram_fingerings.get_by_metric(metric);
                    let score_mode = if print_details.contains(&metrics::Metric::Bigram(metric)) {
                        ScoreMode::Detailed
                    } else {
                        ScoreMode::SummaryUnsafe
                    };
                    let (opt_details, f_sum, f_sum_ew) =
                        score_bfs(fs, &key_table_matrix, &bigram_table, score_mode);
                    (metric, Measurement::new(opt_details, f_sum, f_sum_ew))
                })
                .collect::<BTreeMap<_, _>>();

            let trigram_measurements = metrics::TrigramMetric::VARIANT_ARRAY
                .iter()
                .map(|&metric| {
                    let fs = trigram_fingerings.get_by_metric(metric);
                    let score_mode = if print_details.contains(&metrics::Metric::Trigram(metric)) {
                        ScoreMode::Detailed
                    } else {
                        ScoreMode::SummaryUnsafe
                    };
                    let (opt_details, f_sum, f_sum_ew) =
                        score_tfs(fs, &key_table_matrix, &trigram_table, score_mode);
                    (metric, Measurement::new(opt_details, f_sum, f_sum_ew))
                })
                .collect::<BTreeMap<_, _>>();

            let (uf_sum, uf_sum_ew) = score_ufs_without_details_unsafe(
                unigram_fingerings.get(),
                &key_table_matrix,
                &unigram_table,
            );

            let (bf_sum, bf_sum_ew) = score_bfs_without_details_unsafe(
                bigram_fingerings.get(),
                &key_table_matrix,
                &bigram_table,
            );

            let (tf_sum, tf_sum_ew) = score_tfs_without_details_unsafe(
                trigram_fingerings.get(),
                &key_table_matrix,
                &trigram_table,
            );

            Record {
                key_table_matrix,
                unigram_measurements,
                bigram_measurements,
                trigram_measurements,
                uf_sum,
                uf_sum_ew,
                bf_sum,
                bf_sum_ew,
                tf_sum,
                tf_sum_ew,
            }
        })
        .collect();

    // Sorting

    sort_records(&mut records, &sort_rules, weight);

    // Filtering

    let records = filter_records(records, &filters, weight)?;

    // Selecting

    let records = select_records(records, opt_max_records, opt_index)?;

    // Printing

    let unigram_table_sum = sum_ngram_table(unigram_table.as_ref());
    let bigram_table_sum = sum_ngram_table(bigram_table.as_ref());
    let trigram_table_sum = sum_ngram_table(trigram_table.as_ref());
    let total_selected_records = records.len();

    let opt_metadata = print_metadata
        .unwrap_or(total_permutations > 1)
        .then(|| Metadata {
            unigram_table_sum,
            bigram_table_sum,
            trigram_table_sum,
            goal,
            metric,
            tolerance,
            weight,
            total_permutations,
            elapsed_duration,
            score,
            total_records,
            total_unique_records,
            total_selected_records,
        });

    match format {
        Format::Json => {
            if let Some(metadata) = opt_metadata {
                write_json_flatten_primitive_arrays::<2, _>(
                    &mut stdout,
                    &Value::from(&metadata),
                    0,
                )?;
                writeln!(stdout)?;
            }
            write_records_json(
                &mut stdout,
                records.into_iter(),
                Some(total_selected_records),
                print_summaries,
                print_perc,
            )
        }
        Format::Text => {
            if let Some(metadata) = opt_metadata {
                writeln!(stdout)?;
                metadata.write_styled(&mut stdout)?;
            }
            let unigram_table_normalized = match unigram_table.iter().copied().max() {
                None | Some(0) => [0.0; 1 << 8],
                Some(max) => array::from_fn(|i| unigram_table[i] as f64 / max as f64),
            };
            write_records_text(
                &mut stdout,
                records.into_iter(),
                (total_selected_records > 1).then(|| total_selected_records),
                unigram_table_normalized,
                print_summaries,
                print_perc,
            )
        }
    }?;

    Ok(())
}
