use core::cmp::Reverse;

use crate::{scores::Score, weights::Weight};

pub struct Measurement<K> {
    pub opt_details: Option<Vec<Score<K>>>,
    pub sum: u64,
    pub sum_ew: u64,
}

impl<K> Measurement<K> {
    pub fn new(opt_details: Option<Vec<Score<K>>>, sum: u64, sum_ew: u64) -> Self {
        Self {
            opt_details,
            sum,
            sum_ew,
        }
    }

    pub fn retain_non_zero_details(&mut self) {
        if let Some(details) = self.opt_details.as_mut() {
            details.retain(|score| !score.is_zero());
        }
    }

    pub fn sort_details(&mut self, weight: Weight) {
        if let Some(details) = self.opt_details.as_mut() {
            use Weight::*;
            match weight {
                Effort => details.sort_by_key(|score| Reverse(score.value_ew)),
                Raw => details.sort_by_key(|score| Reverse(score.value)),
            }
        }
    }

    pub fn sum_by_weight(&self, weight: Weight) -> u64 {
        use Weight::*;
        match weight {
            Effort => self.sum_ew,
            Raw => self.sum,
        }
    }
}
