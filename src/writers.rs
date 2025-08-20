use core::{fmt::Display, iter, time::Duration};

use std::{collections::BTreeMap, io, sync::LazyLock};

use serde_json::{Value, json};

use termcolor::{Color, ColorSpec, WriteColor};

use crate::{
    json::write_json_flatten_primitive_arrays,
    keys::KeyTable,
    records::{DetailRow, Record, SummaryRow},
    ui::{colors::hsv_to_rgb, progress::create_progress_bar, styles::WriteStyled},
    util::{
        format::format_perc,
        math::{calculate_frac, crop_matrix},
        time::format_seconds_f64,
    },
};

// Indices

pub static STYLE_INDEX: LazyLock<ColorSpec> = LazyLock::new(|| {
    let mut color_spec = ColorSpec::new();
    color_spec.set_bold(true);
    color_spec.set_underline(true);
    color_spec
});

pub fn write_index(writer: &mut dyn WriteColor, s: &str) -> io::Result<()> {
    writer.set_color(&STYLE_INDEX)?;
    writeln!(writer, "{}", s)?;
    writer.reset()
}

// Matrices

pub static STYLE_NONE: LazyLock<ColorSpec> = LazyLock::new(|| {
    let mut color_spec = ColorSpec::new();
    color_spec.set_dimmed(true);
    color_spec
});

pub static STYLE_SPACE: LazyLock<ColorSpec> = LazyLock::new(|| {
    let mut color_spec = ColorSpec::new();
    color_spec
        .set_bg(Some(Color::White))
        .set_fg(Some(Color::Black));
    color_spec
});

pub static STYLE_SUBSTITUTION: LazyLock<ColorSpec> = LazyLock::new(|| {
    let mut color_spec = ColorSpec::new();
    color_spec.set_bold(true);
    color_spec.set_intense(true);
    color_spec
        .set_bg(Some(Color::Red))
        .set_fg(Some(Color::White));
    color_spec
});

pub static STYLE_UNPRINTABLE: LazyLock<ColorSpec> = LazyLock::new(|| {
    let mut color_spec = ColorSpec::new();
    color_spec.set_dimmed(true);
    color_spec
});

pub fn is_printable(byte: u8) -> bool {
    (0x20..=0x7E).contains(&byte)
}

pub fn write_matrix<const C: usize, const R: usize>(
    writer: &mut dyn WriteColor,
    matrix: &[[u8; C]; R],
    opt_crop_rect_trbl: Option<(usize, usize, usize, usize)>,
    saturation_map: &[f64; 1 << 8],
) -> io::Result<()> {
    const CHAR_UNKNOWN: char = '?';
    const HUE: f32 = 0.0;
    const VALUE_MIN: f32 = 0.75;
    let mut color_spec = ColorSpec::new();
    let (top, right, bottom, left) = opt_crop_rect_trbl.unwrap_or((0, 0, 0, 0));
    for row in top..R.saturating_sub(bottom) {
        for col in left..C.saturating_sub(right) {
            let byte = matrix[row][col];
            match byte {
                0 => {
                    writer.set_color(&STYLE_NONE)?;
                    write!(writer, " ")
                }
                1..=3 => {
                    writer.set_color(&STYLE_SUBSTITUTION)?;
                    write!(writer, "{}", (b'0' + byte) as char)
                }
                b' ' => {
                    writer.set_color(&STYLE_SPACE)?;
                    write!(writer, " ")
                }
                _ if is_printable(byte) => {
                    let s = saturation_map[byte as usize] as f32;
                    let v = VALUE_MIN + s * (1.0 - VALUE_MIN);
                    let (r, g, b) = hsv_to_rgb(HUE, s, v);
                    color_spec.set_fg(Some(Color::Rgb(r, g, b)));
                    writer.set_color(&color_spec)?;
                    write!(writer, "{}", byte as char)
                }
                _ => {
                    writer.set_color(&STYLE_UNPRINTABLE)?;
                    write!(writer, "{}", CHAR_UNKNOWN)
                }
            }?;
            writer.reset()?;
            write!(writer, " ")?;
        }
        writer.reset()?;
        writeln!(writer)?;
    }
    Ok(())
}

// Percentages

pub static STYLE_PERC: LazyLock<ColorSpec> = LazyLock::new(|| {
    let mut color_spec = ColorSpec::new();
    color_spec.set_italic(true);
    color_spec
});

pub fn write_perc(
    writer: &mut dyn WriteColor,
    decimal_places: usize,
    opt_value: Option<f64>,
) -> io::Result<()> {
    writer.set_color(&STYLE_PERC)?;
    write!(writer, "{}", format_perc(decimal_places, opt_value))?;
    writer.reset()
}

// Progress

pub static STYLE_PERC_COMPLETE: LazyLock<ColorSpec> = LazyLock::new(|| {
    let mut color_spec = ColorSpec::new();
    color_spec.set_fg(Some(Color::Green));
    color_spec.set_intense(true);
    color_spec
});

pub static STYLE_DURATION_COMPLETE: LazyLock<ColorSpec> = LazyLock::new(|| {
    let mut color_spec = ColorSpec::new();
    color_spec.set_fg(Some(Color::Cyan));
    color_spec.set_intense(true);
    color_spec
});

pub static STYLE_DURATION_INCOMPLETE: LazyLock<ColorSpec> = LazyLock::new(|| {
    let mut color_spec = ColorSpec::new();
    color_spec.set_fg(Some(Color::Yellow));
    color_spec.set_intense(true);
    color_spec
});

pub fn write_progress(
    writer: &mut dyn WriteColor,
    n: u64,
    opt_total: Option<u64>,
    opt_duration_complete: Option<Duration>,
    estimate_duration_incomplete: bool,
    decimal_places: usize,
    opt_carriage_width: Option<usize>,
    opt_progress_bar_width: Option<usize>,
) -> io::Result<()> {
    const CARRIAGE_WIDTH: usize = 120;
    const PROGRESS_BAR_WIDTH: usize = 20;
    let carriage_width = opt_carriage_width.unwrap_or(CARRIAGE_WIDTH);
    let progress_bar_width = opt_progress_bar_width.unwrap_or(PROGRESS_BAR_WIDTH);
    write!(writer, "\r{:<width$}\r", "", width = carriage_width)?;
    let opt_frac_complete = opt_total.and_then(|t| calculate_frac(n, t));
    if let Some(frac_complete) = opt_frac_complete {
        write!(
            writer,
            "[{}]  ",
            create_progress_bar(progress_bar_width, frac_complete as f32)
        )?;
        let perc_complete = frac_complete * 100.0;
        writer.set_color(&STYLE_PERC_COMPLETE)?;
        write!(writer, "{:.*}%  ", decimal_places, perc_complete)?;
        writer.reset()?;
    }
    write!(writer, "{}", n)?;
    if let Some(total) = opt_total {
        write!(writer, " / {}", total)?;
    }
    if let Some(duration_complete) = opt_duration_complete {
        writer.set_color(&STYLE_DURATION_COMPLETE)?;
        let duration_complete_seconds = duration_complete.as_secs_f64();
        write!(
            writer,
            "  {}",
            format_seconds_f64(duration_complete_seconds, decimal_places)
        )?;
        writer.reset()?;
        if estimate_duration_incomplete {
            if let Some(frac_complete) = opt_frac_complete {
                if frac_complete > 0.0 {
                    let estimated_total_seconds = duration_complete_seconds / frac_complete;
                    let estimated_remaining_seconds =
                        estimated_total_seconds - duration_complete_seconds;
                    writer.set_color(&STYLE_DURATION_INCOMPLETE)?;
                    write!(
                        writer,
                        "  (~ {} remaining)",
                        format_seconds_f64(estimated_remaining_seconds, decimal_places)
                    )?;
                    writer.reset()?;
                }
            }
        }
    }
    writer.flush()
}

// Records

const TOTALS: &str = "TOTALS";

pub fn write_detail_row_json<K: Display>(detail_row: &DetailRow<K>, print_perc: bool) -> Value {
    let mut raw = vec![Value::from(detail_row.value), Value::from(detail_row.cum)];
    if print_perc {
        raw.push(Value::from(detail_row.value_as_perc_measurement));
        raw.push(Value::from(detail_row.cum_as_perc_measurement));
        raw.push(Value::from(detail_row.value_as_perc_record));
        raw.push(Value::from(detail_row.cum_as_perc_record));
    }
    let mut effort = vec![
        Value::from(detail_row.value_ew),
        Value::from(detail_row.cum_ew),
    ];
    if print_perc {
        effort.push(Value::from(detail_row.value_ew_as_perc_measurement));
        effort.push(Value::from(detail_row.cum_ew_as_perc_measurement));
        effort.push(Value::from(detail_row.value_ew_as_perc_record));
        effort.push(Value::from(detail_row.cum_ew_as_perc_record));
    }
    Value::Array(vec![
        Value::from(detail_row.key.to_string()),
        Value::Array(raw),
        Value::Array(effort),
    ])
}

pub fn write_detail_row_text<K: WriteStyled>(
    writer: &mut dyn WriteColor,
    detail_row: &DetailRow<K>,
    decimal_places: usize,
    print_perc: bool,
) -> io::Result<()> {
    detail_row.key.write_styled(writer)?;
    write!(writer, ", ")?;
    write!(writer, "{}", detail_row.value)?;
    write!(writer, ", ")?;
    write!(writer, "{}", detail_row.cum)?;
    if print_perc {
        write!(writer, ", ")?;
        write_perc(writer, decimal_places, detail_row.value_as_perc_measurement)?;
        write!(writer, ", ")?;
        write_perc(writer, decimal_places, detail_row.cum_as_perc_measurement)?;
        write!(writer, ", ")?;
        write_perc(writer, decimal_places, detail_row.value_as_perc_record)?;
        write!(writer, ", ")?;
        write_perc(writer, decimal_places, detail_row.cum_as_perc_record)?;
    }
    write!(writer, ", ")?;
    write!(writer, "{}", detail_row.value_ew)?;
    write!(writer, ", ")?;
    write!(writer, "{}", detail_row.cum_ew)?;
    if print_perc {
        write!(writer, ", ")?;
        write_perc(
            writer,
            decimal_places,
            detail_row.value_ew_as_perc_measurement,
        )?;
        write!(writer, ", ")?;
        write_perc(
            writer,
            decimal_places,
            detail_row.cum_ew_as_perc_measurement,
        )?;
        write!(writer, ", ")?;
        write_perc(writer, decimal_places, detail_row.value_ew_as_perc_record)?;
        write!(writer, ", ")?;
        write_perc(writer, decimal_places, detail_row.cum_ew_as_perc_record)?;
    }
    Ok(())
}

pub fn write_summary_row_json(summary_row: &SummaryRow, print_perc: bool) -> Value {
    let raw = if print_perc {
        Value::Array(vec![
            Value::from(summary_row.sum),
            Value::from(summary_row.sum_as_perc),
        ])
    } else {
        Value::from(summary_row.sum)
    };
    let effort = if print_perc {
        Value::Array(vec![
            Value::from(summary_row.sum_ew),
            Value::from(summary_row.sum_ew_as_perc),
        ])
    } else {
        Value::from(summary_row.sum_ew)
    };
    Value::Array(vec![raw, effort])
}

pub fn write_summary_row_text(
    writer: &mut dyn WriteColor,
    summary_row: &SummaryRow,
    decimal_places: usize,
    print_perc: bool,
) -> io::Result<()> {
    write!(writer, "{}", summary_row.sum)?;
    if print_perc {
        write!(writer, ", ")?;
        write_perc(writer, decimal_places, summary_row.sum_as_perc)?;
    }
    write!(writer, ", {}", summary_row.sum_ew)?;
    if print_perc {
        write!(writer, ", ")?;
        write_perc(writer, decimal_places, summary_row.sum_ew_as_perc)?;
    }
    Ok(())
}

pub fn write_record_json(
    opt_index_and_total: Option<(usize, usize)>,
    record: Record,
    print_summaries: bool,
    print_perc: bool,
) -> Value {
    let key_table = KeyTable::from_byte_matrix(&record.key_table_matrix);
    let key_table_json: Value = (&key_table).into();
    let unigram_details_json = record
        .unigram_measurements
        .iter()
        .filter_map(|(metric, _)| {
            record.iter_unigram_details(*metric).map(|detail_rows| {
                (
                    metric.to_string(),
                    Value::Array(
                        detail_rows
                            .map(|detail_row| write_detail_row_json(&detail_row, print_perc))
                            .collect(),
                    ),
                )
            })
        })
        .collect::<BTreeMap<_, _>>();
    let bigram_details_json = record
        .bigram_measurements
        .iter()
        .filter_map(|(metric, _)| {
            record.iter_bigram_details(*metric).map(|detail_rows| {
                (
                    metric.to_string(),
                    Value::Array(
                        detail_rows
                            .map(|detail_row| write_detail_row_json(&detail_row, print_perc))
                            .collect(),
                    ),
                )
            })
        })
        .collect::<BTreeMap<_, _>>();
    let trigram_details_json = record
        .trigram_measurements
        .iter()
        .filter_map(|(metric, _)| {
            record.iter_trigram_details(*metric).map(|detail_rows| {
                (
                    metric.to_string(),
                    Value::Array(
                        detail_rows
                            .map(|detail_row| write_detail_row_json(&detail_row, print_perc))
                            .collect(),
                    ),
                )
            })
        })
        .collect::<BTreeMap<_, _>>();
    let unigram_summaries_json = print_summaries.then(|| {
        record
            .iter_unigram_summaries()
            .map(|(metric, summary_row)| {
                (
                    metric.to_string(),
                    write_summary_row_json(&summary_row, print_perc),
                )
            })
            .chain(iter::once((
                TOTALS.to_owned(),
                Value::Array(vec![
                    Value::from(record.uf_sum),
                    Value::from(record.uf_sum_ew),
                ]),
            )))
            .collect::<BTreeMap<_, _>>()
    });
    let bigram_summaries_json = print_summaries.then(|| {
        record
            .iter_bigram_summaries()
            .map(|(metric, summary_row)| {
                (
                    metric.to_string(),
                    write_summary_row_json(&summary_row, print_perc),
                )
            })
            .chain(iter::once((
                TOTALS.to_owned(),
                Value::Array(vec![
                    Value::from(record.bf_sum),
                    Value::from(record.bf_sum_ew),
                ]),
            )))
            .collect::<BTreeMap<_, _>>()
    });
    let trigram_summaries_json = print_summaries.then(|| {
        record
            .iter_trigram_summaries()
            .map(|(metric, summary_row)| {
                (
                    metric.to_string(),
                    write_summary_row_json(&summary_row, print_perc),
                )
            })
            .chain(iter::once((
                TOTALS.to_owned(),
                Value::Array(vec![
                    Value::from(record.tf_sum),
                    Value::from(record.tf_sum_ew),
                ]),
            )))
            .collect::<BTreeMap<_, _>>()
    });
    json!({
        "index": opt_index_and_total.map(|(index, _total)| index),
        "key_table": key_table_json,
        "measurements": {
            "unigram": {
                "details": (!unigram_details_json.is_empty()).then_some(unigram_details_json),
                "summaries": unigram_summaries_json,
            },
            "bigram": {
                "details": (!bigram_details_json.is_empty()).then_some(bigram_details_json),
                "summaries": bigram_summaries_json,
            },
            "trigram":  {
                "details": (!trigram_details_json.is_empty()).then_some(trigram_details_json),
                "summaries": trigram_summaries_json,
            },
        },
    })
}

pub fn write_record_text(
    writer: &mut dyn WriteColor,
    opt_index_and_total: Option<(usize, usize)>,
    record: Record,
    unigram_table_normalized: [f64; 1 << 8],
    print_summaries: bool,
    print_perc: bool,
) -> io::Result<()> {
    const DECIMAL_PLACES: usize = 3;
    if let Some((index, total)) = opt_index_and_total {
        write_index(writer, &format!("{} / {}", index, total))?;
        writeln!(writer)?;
    }
    write_matrix(
        writer,
        &record.key_table_matrix,
        Some(crop_matrix(&record.key_table_matrix, |b| is_printable(*b))),
        &unigram_table_normalized,
    )?;
    for metric in record.unigram_measurements.keys() {
        if let Some(detail_rows) = record.iter_unigram_details(*metric) {
            writeln!(writer)?;
            write_title(writer, &format!("{} {}:", metric, metric.goal()))?;
            for detail_row in detail_rows {
                write_detail_row_text(writer, &detail_row, DECIMAL_PLACES, print_perc)?;
                writeln!(writer)?;
            }
        }
    }
    for metric in record.bigram_measurements.keys() {
        if let Some(detail_rows) = record.iter_bigram_details(*metric) {
            writeln!(writer)?;
            write_title(writer, &format!("{} {}:", metric, metric.goal()))?;
            for detail_row in detail_rows {
                write_detail_row_text(writer, &detail_row, DECIMAL_PLACES, print_perc)?;
                writeln!(writer)?;
            }
        }
    }
    for metric in record.trigram_measurements.keys() {
        if let Some(detail_rows) = record.iter_trigram_details(*metric) {
            writeln!(writer)?;
            write_title(writer, &format!("{} {}:", metric, metric.goal()))?;
            for detail_row in detail_rows {
                write_detail_row_text(writer, &detail_row, DECIMAL_PLACES, print_perc)?;
                writeln!(writer)?;
            }
        }
    }
    if print_summaries && !record.unigram_measurements.is_empty() {
        writeln!(writer)?;
        write_title(writer, "Unigram summaries:")?;
        for (metric, summary_row) in record.iter_unigram_summaries() {
            metric.write_styled(writer)?;
            write!(writer, " {}: ", metric.goal())?;
            write_summary_row_text(writer, &summary_row, DECIMAL_PLACES, print_perc)?;
            writeln!(writer)?;
        }
        write!(
            writer,
            "{}: {}, {}",
            TOTALS, record.uf_sum, record.uf_sum_ew
        )?;
        writeln!(writer)?;
    }
    if print_summaries && !record.bigram_measurements.is_empty() {
        writeln!(writer)?;
        write_title(writer, "Bigram summaries:")?;
        for (metric, summary_row) in record.iter_bigram_summaries() {
            metric.write_styled(writer)?;
            write!(writer, " {}: ", metric.goal())?;
            write_summary_row_text(writer, &summary_row, DECIMAL_PLACES, print_perc)?;
            writeln!(writer)?;
        }
        write!(
            writer,
            "{}: {}, {}",
            TOTALS, record.bf_sum, record.bf_sum_ew
        )?;
        writeln!(writer)?;
    }
    if print_summaries && !record.trigram_measurements.is_empty() {
        writeln!(writer)?;
        write_title(writer, "Trigram summaries:")?;
        for (metric, summary_row) in record.iter_trigram_summaries() {
            metric.write_styled(writer)?;
            write!(writer, " {}: ", metric.goal())?;
            write_summary_row_text(writer, &summary_row, DECIMAL_PLACES, print_perc)?;
            writeln!(writer)?;
        }
        write!(
            writer,
            "{}: {}, {}",
            TOTALS, record.tf_sum, record.tf_sum_ew
        )?;
        writeln!(writer)?;
    }
    Ok(())
}

pub fn write_records_json(
    writer: &mut dyn WriteColor,
    records: impl Iterator<Item = Record>,
    opt_total: Option<usize>,
    print_summaries: bool,
    print_perc: bool,
) -> io::Result<()> {
    for (i, record) in records.enumerate() {
        let record_json = write_record_json(
            opt_total.map(|total| (i + 1, total)),
            record,
            print_summaries,
            print_perc,
        );
        write_json_flatten_primitive_arrays::<2, _>(writer, &record_json, 0)?;
        writeln!(writer)?;
        writer.flush()?;
    }
    Ok(())
}

pub fn write_records_text(
    writer: &mut dyn WriteColor,
    records: impl Iterator<Item = Record>,
    opt_total: Option<usize>,
    unigram_table_normalized: [f64; 1 << 8],
    print_summaries: bool,
    print_perc: bool,
) -> io::Result<()> {
    for (i, record) in records.into_iter().enumerate() {
        writeln!(writer)?;
        write_record_text(
            writer,
            opt_total.map(|total| (i + 1, total)),
            record,
            unigram_table_normalized,
            print_summaries,
            print_perc,
        )?;
        writer.flush()?;
    }
    Ok(())
}

// Titles

pub static STYLE_TITLE: LazyLock<ColorSpec> = LazyLock::new(|| {
    let mut color_spec = ColorSpec::new();
    color_spec.set_underline(true);
    color_spec
});

pub fn write_title(writer: &mut dyn WriteColor, s: &str) -> io::Result<()> {
    writer.set_color(&STYLE_TITLE)?;
    writeln!(writer, "{}", s)?;
    writer.reset()
}
