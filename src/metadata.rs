use core::{
    fmt::{self, Display},
    time::Duration,
};

use std::io;

use serde_json::{Value, json};

use termcolor::WriteColor;

use crate::{goals::Goal, metrics::Metric, ui::styles::WriteStyled, weights::Weight};

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct Metadata {
    pub unigram_table_sum: u64,
    pub bigram_table_sum: u64,
    pub trigram_table_sum: u64,
    pub goal: Goal,
    pub metric: Metric,
    pub tolerance: f64,
    pub weight: Weight,
    pub total_permutations: u64,
    pub elapsed_duration: Duration,
    pub score: u64,
    pub truncated: bool,
    pub total_records: usize,
    pub total_unique_records: usize,
    pub total_selected_records: usize,
}

impl Metadata {
    pub fn efficiency(&self) -> Option<Duration> {
        if self.total_permutations == 0 {
            None
        } else {
            Some(Duration::from_secs_f64(
                self.elapsed_duration.as_secs_f64() / self.total_permutations as f64,
            ))
        }
    }
}

impl Display for Metadata {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<&Metadata> for Value {
    fn from(value: &Metadata) -> Self {
        // TODO add tolerance
        json!({
            "unigram_table_sum": value.unigram_table_sum,
            "bigram_table_sum": value.bigram_table_sum,
            "trigram_table_sum": value.trigram_table_sum,
            "goal": value.goal.to_string().to_lowercase(),
            "metric": value.metric.to_string().to_lowercase(),
            "weight": value.weight.to_string().to_lowercase(),
            "total_permutations": value.total_permutations,
            "elapsed_duration": format_duration(value.elapsed_duration),
            "efficiency": format_opt_duration(value.efficiency()),
            "score": value.score,
            "truncated": value.truncated,
            "total_records": value.total_records,
            "total_unique_records": value.total_unique_records,
            "total_selected_records": value.total_selected_records
        })
    }
}

impl WriteStyled for Metadata {
    fn write_styled(&self, writer: &mut dyn WriteColor) -> io::Result<()> {
        // TODO add tolerance
        writeln!(
            writer,
            "unigram table sum:      {}\n\
             bigram table sum:       {}\n\
             trigram table sum:      {}\n\
             goal:                   {}\n\
             metric:                 {}\n\
             weight:                 {}\n\
             total permutations:     {}\n\
             elapsed duration:       {:?}\n\
             efficiency:             {} / permutation\n\
             score:                  {}\n\
             truncated:              {}\n\
             total records:          {}\n\
             total unique records:   {}\n\
             total selected records: {}",
            self.unigram_table_sum,
            self.bigram_table_sum,
            self.trigram_table_sum,
            self.goal.to_string().to_lowercase(),
            self.metric.to_string().to_lowercase(),
            self.weight.to_string().to_lowercase(),
            self.total_permutations,
            self.elapsed_duration,
            format_opt_duration(self.efficiency()),
            self.score,
            self.truncated,
            self.total_records,
            self.total_unique_records,
            self.total_selected_records,
        )
    }
}

fn format_duration(duration: Duration) -> String {
    format!("{:?}", duration)
}

fn format_opt_duration(opt_duration: Option<Duration>) -> String {
    match opt_duration {
        None => String::from("n/a"),
        Some(duration) => format_duration(duration),
    }
}
