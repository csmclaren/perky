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
    pub opt_unigram_table_fpath: Option<&'a Path>,
    pub opt_bigram_table_fpath: Option<&'a Path>,
    pub opt_trigram_table_fpath: Option<&'a Path>,
    pub unigram_table_sum: u64,
    pub bigram_table_sum: u64,
    pub trigram_table_sum: u64,
    pub goal: Goal,
    pub metric: Metric,
    pub tolerance: f64,
    pub weight: Weight,
    pub opt_max_permutations: Option<u64>,
    pub opt_max_records: Option<u32>,
    pub sort_rules: &'a [SortRule],
    pub filters: &'a [Expression],
    pub opt_max_selections: Option<usize>,
    pub opt_index: Option<isize>,
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
        // TODO add tolerance
        // "tolerance": value.tolerance,
        json!({
            "layout_table_fpath": value.layout_table_fpath,
            "key_table_fpath": value.key_table_fpath,
            "opt_unigram_table_fpath": value.opt_unigram_table_fpath,
            "opt_bigram_table_fpath": value.opt_bigram_table_fpath,
            "opt_trigram_table_fpath": value.opt_trigram_table_fpath,
            "unigram_table_sum": value.unigram_table_sum,
            "bigram_table_sum": value.bigram_table_sum,
            "trigram_table_sum": value.trigram_table_sum,
            "goal": value.goal.to_string(),
            "metric": value.metric.to_string(),
            "weight": value.weight.to_string(),
            "opt_max_permutations": value.opt_max_permutations,
            "opt_max_records": value.opt_max_records,
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
            "opt_max_selections": value.opt_max_selections,
            "opt_index": value.opt_index,
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
        // TODO add tolerance
        // tolerance:                  {}\n\
        writeln!(
            writer,
            "layout table fpath:         {:?}\n\
             key table fpath:            {:?}\n\
             opt unigram table fpath:    {}\n\
             opt bigram table fpath:     {}\n\
             opt trigram table fpath:    {}\n\
             unigram table sum:          {}\n\
             bigram table sum:           {}\n\
             trigram table sum:          {}\n\
             goal:                       {}\n\
             metric:                     {}\n\
             weight:                     {}\n\
             opt max permutations:       {}\n\
             opt max records:            {}\n\
             sort rules:                 {}\n\
             filters:                    {}\n\
             opt max selections:         {}\n\
             opt index:                  {}\n\
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
            format_opt_debug(self.opt_unigram_table_fpath),
            format_opt_debug(self.opt_bigram_table_fpath),
            format_opt_debug(self.opt_trigram_table_fpath),
            self.unigram_table_sum,
            self.bigram_table_sum,
            self.trigram_table_sum,
            self.goal.to_string(),
            self.metric.to_string(),
            self.weight.to_string(),
            format_opt_display(self.opt_max_permutations),
            format_opt_display(self.opt_max_records),
            DisplaySlice(self.sort_rules),
            DisplaySlice(self.filters),
            format_opt_display(self.opt_max_selections),
            format_opt_display(self.opt_index),
            self.total_permutations,
            self.permutations_truncated,
            self.total_records,
            self.records_truncated,
            format_duration(self.elapsed_duration),
            format_opt_duration(self.efficiency()),
            self.total_unique_records,
            self.total_selected_records,
        )
    }
}

fn format_opt_debug<T: Debug>(opt_debug: Option<T>) -> String {
    match opt_debug {
        None => String::from("null"),
        Some(t) => format!("{:?}", t),
    }
}

fn format_opt_display<T: Display>(opt_display: Option<T>) -> String {
    match opt_display {
        None => String::from("null"),
        Some(t) => format!("{}", t),
    }
}

fn format_duration(duration: Duration) -> String {
    format!("{:?}", duration)
}

fn format_opt_duration(opt_duration: Option<Duration>) -> String {
    format_opt_debug(opt_duration)
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
