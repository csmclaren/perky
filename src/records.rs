use core::{cmp::Ordering, fmt::Display};

use std::collections::{BTreeMap, HashMap};

use crate::{
    expressions::{EvalError, Expression, Value},
    measurements::Measurement,
    metrics::{BigramMetric, Metric, SortDirection, SortRule, TrigramMetric, UnigramMetric},
    ngrams::{BigramKey, TrigramKey, UnigramKey},
    scores::Score,
    util::math::calculate_perc,
    weights::Weight,
};

struct DetailIter<'a, K: Clone> {
    iter: std::slice::Iter<'a, Score<K>>,
    cum: u64,
    cum_ew: u64,
    measurement_sum: u64,
    measurement_sum_ew: u64,
    record_sum: u64,
    record_sum_ew: u64,
}

impl<'a, K: Clone> DetailIter<'a, K> {
    pub fn new(
        measurement: &'a Measurement<K>,
        record_sum: u64,
        record_sum_ew: u64,
    ) -> Option<Self> {
        Some(Self {
            iter: measurement.details_opt.as_deref()?.iter(),
            cum: 0,
            cum_ew: 0,
            measurement_sum: measurement.sum,
            measurement_sum_ew: measurement.sum_ew,
            record_sum,
            record_sum_ew,
        })
    }
}

impl<'a, K: Clone> Iterator for DetailIter<'a, K> {
    type Item = DetailRow<K>;
    fn next(&mut self) -> Option<Self::Item> {
        let score = self.iter.next()?;
        self.cum += score.value;
        self.cum_ew += score.value_ew;
        Some(DetailRow::new(
            score,
            self.cum,
            self.cum_ew,
            self.measurement_sum,
            self.measurement_sum_ew,
            self.record_sum,
            self.record_sum_ew,
        ))
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct DetailRow<K> {
    pub key: K,
    pub value: u64,
    pub cum: u64,
    pub value_as_perc_measurement: Option<f64>,
    pub cum_as_perc_measurement: Option<f64>,
    pub value_as_perc_record: Option<f64>,
    pub cum_as_perc_record: Option<f64>,
    pub value_ew: u64,
    pub cum_ew: u64,
    pub value_ew_as_perc_measurement: Option<f64>,
    pub cum_ew_as_perc_measurement: Option<f64>,
    pub value_ew_as_perc_record: Option<f64>,
    pub cum_ew_as_perc_record: Option<f64>,
}

impl<K: Clone> DetailRow<K> {
    pub fn new(
        score: &Score<K>,
        cum: u64,
        cum_ew: u64,
        measurement_sum: u64,
        measurement_sum_ew: u64,
        record_sum: u64,
        record_sum_ew: u64,
    ) -> Self {
        Self {
            key: score.key.clone(),
            value: score.value,
            cum: cum,
            value_as_perc_measurement: calculate_perc(score.value, measurement_sum),
            cum_as_perc_measurement: calculate_perc(cum, measurement_sum),
            value_as_perc_record: calculate_perc(score.value, record_sum),
            cum_as_perc_record: calculate_perc(cum, record_sum),
            value_ew: score.value_ew,
            cum_ew: cum_ew,
            value_ew_as_perc_measurement: calculate_perc(score.value_ew, measurement_sum_ew),
            cum_ew_as_perc_measurement: calculate_perc(cum_ew, measurement_sum_ew),
            value_ew_as_perc_record: calculate_perc(score.value_ew, record_sum_ew),
            cum_ew_as_perc_record: calculate_perc(cum_ew, record_sum_ew),
        }
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct SummaryRow {
    pub sum: u64,
    pub sum_as_perc: Option<f64>,
    pub sum_ew: u64,
    pub sum_ew_as_perc: Option<f64>,
}

impl SummaryRow {
    pub fn new<K>(measurement: &Measurement<K>, record_sum: u64, record_sum_ew: u64) -> Self {
        Self {
            sum: measurement.sum,
            sum_as_perc: calculate_perc(measurement.sum, record_sum),
            sum_ew: measurement.sum_ew,
            sum_ew_as_perc: calculate_perc(measurement.sum_ew, record_sum_ew),
        }
    }
}

pub struct Record {
    pub key_table_matrix: [[u8; 16]; 8],
    pub unigram_measurements: BTreeMap<UnigramMetric, Measurement<UnigramKey>>,
    pub bigram_measurements: BTreeMap<BigramMetric, Measurement<BigramKey>>,
    pub trigram_measurements: BTreeMap<TrigramMetric, Measurement<TrigramKey>>,
    pub uf_sum: u64,
    pub uf_sum_ew: u64,
    pub bf_sum: u64,
    pub bf_sum_ew: u64,
    pub tf_sum: u64,
    pub tf_sum_ew: u64,
}

impl Record {
    pub fn build_symbol_table(&self, weight: Weight) -> HashMap<String, Value> {
        fn iter_pairs<'a, T: Display, U>(
            map: &'a BTreeMap<T, Measurement<U>>,
            denominator: u64,
            weight: Weight,
        ) -> impl 'a + Iterator<Item = (String, Value)> {
            map.iter().filter_map(move |(metric, measurement)| {
                // NOTE
                // if a percentage cannot be calculated, it's because the denominator was zero,
                // and that would only be the case if the n-gram table for that type of metric
                // contained no n-gram data. in this case, the symbol has no value and is not
                // added to the symbol table.
                //
                // TODO
                // attempting to use this symbol will result in an 'undefined variable' error,
                // which is somewhat misleading as to its root cause. perhaps this could be
                // improved.
                //
                calculate_perc(measurement.sum_by_weight(weight), denominator)
                    .map(|perc| (metric.to_string().to_lowercase(), Value::Number(perc)))
            })
        }

        let (unigram_denominator, bigram_denominator, trigram_denominator) = match weight {
            Weight::Effort => (self.uf_sum_ew, self.bf_sum_ew, self.tf_sum_ew),
            Weight::Raw => (self.uf_sum, self.bf_sum, self.tf_sum),
        };
        let mut symbol_table = HashMap::with_capacity(
            self.unigram_measurements.len()
                + self.bigram_measurements.len()
                + self.trigram_measurements.len(),
        );
        symbol_table.extend(iter_pairs(
            &self.unigram_measurements,
            unigram_denominator,
            weight,
        ));
        symbol_table.extend(iter_pairs(
            &self.bigram_measurements,
            bigram_denominator,
            weight,
        ));
        symbol_table.extend(iter_pairs(
            &self.trigram_measurements,
            trigram_denominator,
            weight,
        ));
        symbol_table
    }

    pub fn iter_unigram_details(
        &self,
        metric: UnigramMetric,
    ) -> Option<impl '_ + Iterator<Item = DetailRow<UnigramKey>>> {
        DetailIter::new(
            self.unigram_measurements.get(&metric)?,
            self.uf_sum,
            self.uf_sum_ew,
        )
    }

    pub fn iter_bigram_details(
        &self,
        metric: BigramMetric,
    ) -> Option<impl '_ + Iterator<Item = DetailRow<BigramKey>>> {
        DetailIter::new(
            self.bigram_measurements.get(&metric)?,
            self.bf_sum,
            self.bf_sum_ew,
        )
    }

    pub fn iter_trigram_details(
        &self,
        metric: TrigramMetric,
    ) -> Option<impl '_ + Iterator<Item = DetailRow<TrigramKey>>> {
        DetailIter::new(
            self.trigram_measurements.get(&metric)?,
            self.tf_sum,
            self.tf_sum_ew,
        )
    }

    pub fn iter_unigram_summaries(&self) -> impl '_ + Iterator<Item = (UnigramMetric, SummaryRow)> {
        self.unigram_measurements
            .iter()
            .map(move |(metric, measurement)| {
                (
                    *metric,
                    SummaryRow::new(measurement, self.uf_sum, self.uf_sum_ew),
                )
            })
    }

    pub fn iter_bigram_summaries(&self) -> impl '_ + Iterator<Item = (BigramMetric, SummaryRow)> {
        self.bigram_measurements
            .iter()
            .map(move |(metric, measurement)| {
                (
                    *metric,
                    SummaryRow::new(measurement, self.bf_sum, self.bf_sum_ew),
                )
            })
    }

    pub fn iter_trigram_summaries(&self) -> impl '_ + Iterator<Item = (TrigramMetric, SummaryRow)> {
        self.trigram_measurements
            .iter()
            .map(move |(metric, measurement)| {
                (
                    *metric,
                    SummaryRow::new(measurement, self.tf_sum, self.tf_sum_ew),
                )
            })
    }

    pub fn normalize(&mut self, weight: Weight) {
        for measurement in self.unigram_measurements.values_mut() {
            measurement.retain_non_zero_details();
            measurement.sort_details(weight);
        }
        for measurement in self.bigram_measurements.values_mut() {
            measurement.retain_non_zero_details();
            measurement.sort_details(weight);
        }
        for measurement in self.trigram_measurements.values_mut() {
            measurement.retain_non_zero_details();
            measurement.sort_details(weight);
        }
    }

    pub fn sum(&self, metric: Metric, weight: Weight) -> Option<u64> {
        use Metric::*;
        match &metric {
            Unigram(metric) => self
                .unigram_measurements
                .get(metric)
                .map(|measurement| measurement.sum_by_weight(weight)),
            Bigram(metric) => self
                .bigram_measurements
                .get(metric)
                .map(|measurement| measurement.sum_by_weight(weight)),
            Trigram(metric) => self
                .trigram_measurements
                .get(metric)
                .map(|measurement| measurement.sum_by_weight(weight)),
        }
    }
}

pub fn filter_records(
    records: Vec<Record>,
    filters: &[Expression],
    weight: Weight,
) -> Result<Vec<Record>, EvalError> {
    records
        .into_iter()
        .filter_map(|mut record| {
            if !filters.is_empty() {
                let symbol_table = record.build_symbol_table(weight);
                for filter in filters {
                    use Value::*;
                    match filter.evaluate(&symbol_table) {
                        Ok(Number(n)) if n == 0.0 => return None,
                        Ok(Boolean(b)) if !b => return None,
                        Ok(_) => continue,
                        Err(e) => return Some(Err(e)),
                    }
                }
            }
            record.normalize(weight);
            Some(Ok(record))
        })
        .collect::<Result<Vec<_>, _>>()
}

pub fn select_records(
    mut records: Vec<Record>,
    max_selections: Option<usize>,
    index: Option<isize>,
) -> Result<Vec<Record>, String> {
    if let Some(max_selections) = max_selections {
        records.truncate(max_selections);
    }
    if let Some(index) = index {
        let length = records.len() as isize;
        let i = if index < 0 { length + index } else { index };
        if i < 0 || i >= length {
            return Err(format!(
                "Index {} out of bounds for {} entries",
                index, length
            ));
        }
        records.swap(0, i as usize);
        records.truncate(1);
    }
    Ok(records)
}

pub fn sort_records(records: &mut [Record], sort_rules: &[SortRule], weight: Weight) {
    records.sort_by(|a, b| {
        use Ordering::*;
        use SortDirection::*;
        for sort_rule in sort_rules {
            let ordering = a
                .sum(sort_rule.metric, weight)
                .cmp(&b.sum(sort_rule.metric, weight));
            let ordering = match sort_rule.sort_direction {
                Ascending => ordering,
                Descending => ordering.reverse(),
            };
            if ordering != Equal {
                return ordering;
            }
        }
        Equal
    });
}
