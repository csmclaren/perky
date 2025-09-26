use core::{
    fmt::{self, Display},
    time::Duration,
};

use std::{fmt::Debug, io, path::Path};

use serde_json::{Value, json};

use termcolor::WriteColor;

use crate::{
    expressions::Expression,
    goals::Goal,
    metrics::{Metric, SortRule},
    ui::styles::WriteStyled,
    weights::Weight,
};

#[derive(Debug)]
pub struct Metadata<'a> {
    pub layout_table_fpath: &'a Path,
    pub key_table_fpath: &'a Path,
    pub unigram_table_fpath_opt: Option<&'a Path>,
    pub bigram_table_fpath_opt: Option<&'a Path>,
    pub trigram_table_fpath_opt: Option<&'a Path>,
    pub unigram_table_sum: u64,
    pub bigram_table_sum: u64,
    pub trigram_table_sum: u64,
    pub goal: Goal,
    pub metric: Metric,
    pub tolerance: f64,
    pub weight: Weight,
    pub max_permutations_opt: Option<u64>,
    pub max_records_opt: Option<u32>,
    pub sort_rules: &'a [SortRule],
    pub filters: &'a [Expression],
    pub max_selections_opt: Option<usize>,
    pub index_opt: Option<isize>,
    pub total_permutations: u64,
    pub permutations_truncated: bool,
    pub total_records: usize,
    pub records_truncated: bool,
    pub elapsed_duration: Duration,
    pub total_unique_records: usize,
    pub total_selected_records: usize,
}

impl Metadata<'_> {
    pub fn efficiency(&self) -> Option<Duration> {
        (self.total_permutations != 0).then(|| {
            Duration::from_secs_f64(
                self.elapsed_duration.as_secs_f64() / self.total_permutations as f64,
            )
        })
    }
}

impl Display for Metadata<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<&Metadata<'_>> for Value {
    fn from(value: &Metadata<'_>) -> Self {
        json!({
            "layout_table_fpath": value.layout_table_fpath,
            "key_table_fpath": value.key_table_fpath,
            "unigram_table_fpath": value.unigram_table_fpath_opt,
            "bigram_table_fpath": value.bigram_table_fpath_opt,
            "trigram_table_fpath": value.trigram_table_fpath_opt,
            "unigram_table_sum": value.unigram_table_sum,
            "bigram_table_sum": value.bigram_table_sum,
            "trigram_table_sum": value.trigram_table_sum,
            "goal": value.goal.to_string(),
            "metric": value.metric.to_string(),
            "tolerance": value.tolerance,
            "weight": value.weight.to_string(),
            "max_permutations": value.max_permutations_opt,
            "max_records": value.max_records_opt,
            "sort_rules": value
                .sort_rules
                .iter()
                .map(|sort_rule| sort_rule.to_string())
                .collect::<Vec<String>>(),
            "filters": value
                .filters
                .iter()
                .map(|expression| expression.to_string())
                .collect::<Vec<String>>(),
            "max_selections": value.max_selections_opt,
            "index": value.index_opt,
            "total_permutations": value.total_permutations,
            "permutations_truncated": value.permutations_truncated,
            "total_records": value.total_records,
            "records_truncated": value.records_truncated,
            "elapsed_duration": value.elapsed_duration,
            "efficiency": value.efficiency(),
            "total_unique_records": value.total_unique_records,
            "total_selected_records": value.total_selected_records
        })
    }
}

impl WriteStyled for Metadata<'_> {
    fn write_styled(&self, writer: &mut dyn WriteColor) -> io::Result<()> {
        writeln!(
            writer,
            "layout table fpath:         {:?}\n\
             key table fpath:            {:?}\n\
             unigram table fpath:        {}\n\
             bigram table fpath:         {}\n\
             trigram table fpath:        {}\n\
             unigram table sum:          {}\n\
             bigram table sum:           {}\n\
             trigram table sum:          {}\n\
             goal:                       {}\n\
             metric:                     {}\n\
             tolerance:                  {}\n\
             weight:                     {}\n\
             max permutations:           {}\n\
             max records:                {}\n\
             sort rules:                 {}\n\
             filters:                    {}\n\
             max selections:             {}\n\
             index:                      {}\n\
             total permutations:         {}\n\
             permutations truncated:     {}\n\
             total records:              {}\n\
             records truncated:          {}\n\
             elapsed duration:           {}\n\
             efficiency:                 {} / permutation\n\
             total unique records:       {}\n\
             total selected records:     {}",
            self.layout_table_fpath,
            self.key_table_fpath,
            format_debug_opt(self.unigram_table_fpath_opt),
            format_debug_opt(self.bigram_table_fpath_opt),
            format_debug_opt(self.trigram_table_fpath_opt),
            self.unigram_table_sum,
            self.bigram_table_sum,
            self.trigram_table_sum,
            self.goal.to_string(),
            self.metric.to_string(),
            self.tolerance,
            self.weight.to_string(),
            format_display_opt(self.max_permutations_opt),
            format_display_opt(self.max_records_opt),
            DisplaySlice(self.sort_rules),
            DisplaySlice(self.filters),
            format_display_opt(self.max_selections_opt),
            format_display_opt(self.index_opt),
            self.total_permutations,
            self.permutations_truncated,
            self.total_records,
            self.records_truncated,
            format_duration(self.elapsed_duration),
            format_duration_opt(self.efficiency()),
            self.total_unique_records,
            self.total_selected_records,
        )
    }
}

fn format_debug_opt<T: Debug>(debug_opt: Option<T>) -> String {
    match debug_opt {
        None => String::from("null"),
        Some(t) => format!("{:?}", t),
    }
}

fn format_display_opt<T: Display>(display_opt: Option<T>) -> String {
    match display_opt {
        None => String::from("null"),
        Some(t) => format!("{}", t),
    }
}

fn format_duration(duration: Duration) -> String {
    format!("{:?}", duration)
}

fn format_duration_opt(duration_opt: Option<Duration>) -> String {
    format_debug_opt(duration_opt)
}

struct DisplaySlice<'a, T>(&'a [T]);

impl<'a, T: Display> Display for DisplaySlice<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[")?;
        for (i, item) in self.0.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{item}")?;
        }
        write!(f, "]")
    }
}
