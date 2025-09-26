use core::{error::Error, sync::atomic, time::Duration};

use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
    thread::sleep,
};

use rayon::iter::{IntoParallelIterator, ParallelIterator};

use crate::{
    goals::Goal,
    util::math::{factorial, generate_permutations_to_limit, index_to_permutation_in_place},
};

#[inline]
fn calculate_threshold(goal: Goal, best: u64, tolerance: f64) -> u64 {
    if tolerance == 1.0 {
        best
    } else {
        use Goal::*;
        match goal {
            Max if tolerance == 0.0 => 0,
            Max => ((best as f64) * tolerance).floor() as u64,
            Min if tolerance == 0.0 => u64::MAX,
            Min => ((best as f64) / tolerance).ceil() as u64,
        }
    }
}

#[inline]
fn drop_above_threshold<const C: usize, const R: usize>(
    deque: &mut VecDeque<(u64, u64, [[u8; C]; R])>,
    threshold: u64,
) {
    while let Some((score, _, _)) = deque.front() {
        if *score <= threshold {
            break;
        }
        deque.pop_front();
    }
}

#[inline]
fn drop_below_threshold<const C: usize, const R: usize>(
    deque: &mut VecDeque<(u64, u64, [[u8; C]; R])>,
    threshold: u64,
) {
    while let Some((score, _, _)) = deque.back() {
        if *score >= threshold {
            break;
        }
        deque.pop_back();
    }
}

#[inline]
fn insert_sorted<const C: usize, const R: usize>(
    deque: &mut VecDeque<(u64, u64, [[u8; C]; R])>,
    score: u64,
    index: u64,
    matrix: [[u8; C]; R],
) {
    let pos = match deque.binary_search_by(|(s, i, _)| match s.cmp(&score).reverse() {
        core::cmp::Ordering::Equal => i.cmp(&index),
        other => other,
    }) {
        Ok(i) => i + 1,
        Err(i) => i,
    };
    if pos == deque.len() {
        deque.push_back((score, index, matrix));
    } else {
        deque.insert(pos, (score, index, matrix));
    }
}

#[inline]
fn truncate<const C: usize, const R: usize>(
    deque: &mut VecDeque<(u64, u64, [[u8; C]; R])>,
    goal: Goal,
    max_records_opt: Option<u64>,
) {
    if let Some(max_records) = max_records_opt {
        let max_records = max_records as usize;
        use Goal::*;
        match goal {
            Max => {
                deque.truncate(max_records);
            }
            Min => {
                let length = deque.len();
                if length <= max_records {
                    return;
                }
                deque.drain(..(length - max_records));
            }
        }
    }
}

#[inline]
fn consider_record<const C: usize, const R: usize>(
    matrix: [[u8; C]; R],
    score: u64,
    index: u64,
    goal: Goal,
    tolerance: f64,
    max_records_opt: Option<u64>,
    records: &mut VecDeque<(u64, u64, [[u8; C]; R])>,
    best_score: &mut u64,
    threshold_score: &mut u64,
) {
    use Goal::*;
    match goal {
        Max => {
            if score > *best_score {
                *best_score = score;
                *threshold_score = calculate_threshold(goal, *best_score, tolerance);
                drop_below_threshold(records, *threshold_score);
            }
            if score >= *threshold_score {
                insert_sorted(records, score, index, matrix);
                truncate(records, goal, max_records_opt);
            }
        }
        Min => {
            if score < *best_score {
                *best_score = score;
                *threshold_score = calculate_threshold(goal, *best_score, tolerance);
                drop_above_threshold(records, *threshold_score);
            }
            if score <= *threshold_score {
                insert_sorted(records, score, index, matrix);
                truncate(records, goal, max_records_opt);
            }
        }
    }
}

pub fn convert_vec_opt_to_array<const N: usize, T: Default + Copy>(
    vec_opt: Option<Vec<T>>,
) -> Result<([T; N], usize), Box<dyn Error>> {
    let mut array = [T::default(); N];
    match vec_opt {
        None => Ok((array, 0)),
        Some(vec) => {
            let len = vec.len();
            if len > N {
                Err(format!(
                    "Vector contains {len} elements. expected {N} or fewer"
                ))?
            }
            array[..len].copy_from_slice(&vec);
            Ok((array, len))
        }
    }
}

pub fn permute_and_substitute<const C: usize, const R: usize, const N: usize>(
    matrix: &[[u8; C]; R],
    region1: ([u8; N], usize, &[(usize, usize)]),
    region2: ([u8; N], usize, &[(usize, usize)]),
    region3: ([u8; N], usize, &[(usize, usize)]),
    progress_fn: impl FnMut(u64, bool) -> bool + Send + Sync,
    scoring_fn: impl Fn(&[[u8; C]; R]) -> u64 + Sync,
    goal: Goal,
    tolerance: f64,
    max_permutations_opt: Option<u64>,
    max_records_opt: Option<u32>,
    parallelize: bool,
    sleep_ns: u64,
) -> Result<(u64, bool, Vec<[[u8; C]; R]>, bool), Box<dyn Error>> {
    let max_records_opt = max_records_opt.map(|max_records: u32| max_records as u64 + 1);
    let result = if parallelize {
        permute_and_substitute_parallel(
            &matrix,
            region1,
            region2,
            region3,
            progress_fn,
            scoring_fn,
            goal,
            tolerance,
            max_permutations_opt,
            max_records_opt,
            sleep_ns,
        )
    } else {
        permute_and_substitute_sequential(
            &matrix,
            region1,
            region2,
            region3,
            progress_fn,
            scoring_fn,
            goal,
            tolerance,
            max_permutations_opt,
            max_records_opt,
            sleep_ns,
        )
    };
    result.map(
        |(total_permutations, permutations_truncated, mut records)| {
            let records_truncated = max_records_opt.map_or(false, |max_records| {
                records.len() as u64 >= max_records && records.pop().is_some()
            });
            (
                total_permutations,
                permutations_truncated,
                records,
                records_truncated,
            )
        },
    )
}

fn permute_and_substitute_parallel<const C: usize, const R: usize, const N: usize>(
    matrix: &[[u8; C]; R],
    region1: ([u8; N], usize, &[(usize, usize)]),
    region2: ([u8; N], usize, &[(usize, usize)]),
    region3: ([u8; N], usize, &[(usize, usize)]),
    progress_fn: impl FnMut(u64, bool) -> bool + Send + Sync,
    scoring_fn: impl Fn(&[[u8; C]; R]) -> u64 + Sync,
    goal: Goal,
    tolerance: f64,
    max_permutations_opt: Option<u64>,
    max_records_opt: Option<u64>,
    sleep_ns: u64,
) -> Result<(u64, bool, Vec<[[u8; C]; R]>), Box<dyn Error>> {
    const BATCH: u64 = 1000;
    use Goal::*;
    let initial_score = match goal {
        Max => 0,
        Min => u64::MAX,
    };
    let tolerance = tolerance.clamp(0.0, 1.0);
    let (array1, length1, coordinates1) = region1;
    let (array2, length2, coordinates2) = region2;
    let (array3, length3, coordinates3) = region3;
    let length1 = length1.min(N);
    let length2 = length2.min(N);
    let length3 = length3.min(N);
    let coordinates1 = &coordinates1[..length1];
    let coordinates2 = &coordinates2[..length2];
    let coordinates3 = &coordinates3[..length3];
    let n1 = length1 as u64;
    let n2 = length2 as u64;
    let n3 = length3 as u64;
    let total1 = factorial(n1);
    let total2 = factorial(n2);
    let total3 = factorial(n3);
    let total_permutations = total1.saturating_mul(total2).saturating_mul(total3);
    let max_permutations = max_permutations_opt.unwrap_or(u64::MAX);
    let permutations_truncated = max_permutations < total_permutations;
    let n_permutations = Arc::new(atomic::AtomicU64::new(0));
    let progress_fn = Arc::new(Mutex::new(progress_fn));
    let (records, _best_score, _threshold_score) = (0..total_permutations.min(max_permutations))
        .into_par_iter()
        .fold(
            || {
                (
                    VecDeque::with_capacity(max_records_opt.unwrap_or(0) as usize),
                    initial_score,
                    calculate_threshold(goal, initial_score, tolerance),
                    0u64,
                )
            },
            |(
                mut local_records,
                mut local_best_score,
                mut local_threshold_score,
                mut local_n_permutations,
            ),
             index| {
                let mut matrix = *matrix;
                let mut p1 = [0u8; N];
                let mut p2 = [0u8; N];
                let mut p3 = [0u8; N];
                let index1 = index / (total2 * total3);
                let index2 = (index / total3) % total2;
                let index3 = index % total3;
                index_to_permutation_in_place::<N, u8>(
                    index1,
                    &array1[..length1],
                    &mut p1[..length1],
                );
                index_to_permutation_in_place::<N, u8>(
                    index2,
                    &array2[..length2],
                    &mut p2[..length2],
                );
                index_to_permutation_in_place::<N, u8>(
                    index3,
                    &array3[..length3],
                    &mut p3[..length3],
                );
                if length1 > 0 {
                    for (i, &(r, c)) in coordinates1.iter().enumerate() {
                        matrix[r][c] = p1[i];
                    }
                }
                if length2 > 0 {
                    for (i, &(r, c)) in coordinates2.iter().enumerate() {
                        matrix[r][c] = p2[i];
                    }
                }
                if length3 > 0 {
                    for (i, &(r, c)) in coordinates3.iter().take(length3).enumerate() {
                        matrix[r][c] = p3[i];
                    }
                }
                let score = scoring_fn(&matrix);
                consider_record(
                    matrix,
                    score,
                    index,
                    goal,
                    tolerance,
                    max_records_opt,
                    &mut local_records,
                    &mut local_best_score,
                    &mut local_threshold_score,
                );
                local_n_permutations += 1;
                if local_n_permutations % BATCH == 0 {
                    let current =
                        n_permutations.fetch_add(BATCH, atomic::Ordering::Relaxed) + BATCH;
                    if let Ok(mut progress_fn) = progress_fn.lock() {
                        progress_fn(current, false);
                    }
                    if sleep_ns != 0 {
                        sleep(Duration::from_nanos(sleep_ns));
                    }
                }

                (
                    local_records,
                    local_best_score,
                    local_threshold_score,
                    local_n_permutations,
                )
            },
        )
        .map(
            |(local_records, local_best_score, local_threshold_score, local_n_permutations)| {
                let remaining = local_n_permutations % BATCH;
                if remaining != 0 {
                    n_permutations.fetch_add(remaining, atomic::Ordering::Relaxed);
                }
                (local_records, local_best_score, local_threshold_score)
            },
        )
        .reduce(
            || {
                (
                    VecDeque::with_capacity(max_records_opt.unwrap_or(0) as usize),
                    initial_score,
                    calculate_threshold(goal, initial_score, tolerance),
                )
            },
            |(records_1, best_score_1, threshold_score_1),
             (records_2, best_score_2, threshold_score_2)| {
                let (mut left, mut right, best_score, threshold_score) = match goal {
                    Max => {
                        if best_score_1 >= best_score_2 {
                            (records_1, records_2, best_score_1, threshold_score_1)
                        } else {
                            (records_2, records_1, best_score_2, threshold_score_2)
                        }
                    }
                    Min => {
                        if best_score_1 <= best_score_2 {
                            (records_1, records_2, best_score_1, threshold_score_1)
                        } else {
                            (records_2, records_1, best_score_2, threshold_score_2)
                        }
                    }
                };
                match goal {
                    Max => {
                        drop_below_threshold(&mut left, threshold_score);
                        drop_below_threshold(&mut right, threshold_score);
                    }
                    Min => {
                        drop_above_threshold(&mut left, threshold_score);
                        drop_above_threshold(&mut right, threshold_score);
                    }
                }
                let mut merged: VecDeque<(u64, u64, [[u8; C]; R])> =
                    VecDeque::with_capacity(max_records_opt.unwrap_or(0) as usize);
                let max_records_opt = max_records_opt.map(|max_records| max_records as usize);
                while !left.is_empty() && !right.is_empty() {
                    if let Some(max_records) = max_records_opt {
                        if merged.len() >= max_records {
                            return (merged, best_score, threshold_score);
                        }
                    }
                    let (s1, i1, _) = *left.front().unwrap();
                    let (s2, i2, _) = *right.front().unwrap();
                    if (s1 > s2) || (s1 == s2 && i1 <= i2) {
                        let item = left.pop_front().unwrap();
                        merged.push_back(item);
                    } else {
                        let item = right.pop_front().unwrap();
                        merged.push_back(item);
                    }
                }
                if let Some(max_records) = max_records_opt {
                    while merged.len() < max_records {
                        if let Some(item) = left.pop_front() {
                            merged.push_back(item);
                        } else {
                            break;
                        }
                    }
                    while merged.len() < max_records {
                        if let Some(item) = right.pop_front() {
                            merged.push_back(item);
                        } else {
                            break;
                        }
                    }
                } else {
                    for (s, i, m) in left {
                        merged.push_back((s, i, m));
                    }
                    for (s, i, m) in right {
                        merged.push_back((s, i, m));
                    }
                }
                (merged, best_score, threshold_score)
            },
        );
    let n_permutations = n_permutations.load(atomic::Ordering::Relaxed);
    if let Ok(mut progress_fn) = progress_fn.lock() {
        progress_fn(n_permutations, true);
    }
    let records: Vec<[[u8; C]; R]> = records.into_iter().map(|(_, _, m)| m).collect();
    Ok((n_permutations, permutations_truncated, records))
}

fn permute_and_substitute_sequential<const C: usize, const R: usize, const N: usize>(
    matrix: &[[u8; C]; R],
    region1: ([u8; N], usize, &[(usize, usize)]),
    region2: ([u8; N], usize, &[(usize, usize)]),
    region3: ([u8; N], usize, &[(usize, usize)]),
    mut progress_fn: impl FnMut(u64, bool) -> bool,
    scoring_fn: impl Fn(&[[u8; C]; R]) -> u64,
    goal: Goal,
    tolerance: f64,
    max_permutations_opt: Option<u64>,
    max_records_opt: Option<u64>,
    sleep_ns: u64,
) -> Result<(u64, bool, Vec<[[u8; C]; R]>), Box<dyn Error>> {
    const BATCH: u64 = 1000000;
    use Goal::*;
    let initial_score = match goal {
        Max => 0,
        Min => u64::MAX,
    };
    let tolerance = tolerance.clamp(0.0, 1.0);
    let (array1, length1, coordinates1) = region1;
    let (array2, length2, coordinates2) = region2;
    let (array3, length3, coordinates3) = region3;
    let length1 = length1.min(N);
    let length2 = length2.min(N);
    let length3 = length3.min(N);
    let coordinates1 = &coordinates1[..length1];
    let coordinates2 = &coordinates2[..length2];
    let coordinates3 = &coordinates3[..length3];
    let n1 = length1 as u64;
    let n2 = length2 as u64;
    let n3 = length3 as u64;
    let total1 = factorial(n1);
    let total2 = factorial(n2);
    let total3 = factorial(n3);
    let total_permutations = total1.saturating_mul(total2).saturating_mul(total3);
    let max_permutations = max_permutations_opt.unwrap_or(u64::MAX);
    let permutations_truncated = max_permutations < total_permutations;
    let mut n_permutations = 0u64;
    let mut records: VecDeque<(u64, u64, [[u8; C]; R])> =
        VecDeque::with_capacity(max_records_opt.unwrap_or(0) as usize);
    let mut best_score = initial_score;
    let mut threshold_score = calculate_threshold(goal, best_score, tolerance);
    let mut matrix = *matrix;
    generate_permutations_to_limit::<N, u8>(array1, length1, |p1| {
        generate_permutations_to_limit::<N, u8>(array2, length2, |p2| {
            generate_permutations_to_limit::<N, u8>(array3, length3, |p3| {
                if length1 > 0 {
                    for (i, &(r, c)) in coordinates1.iter().enumerate() {
                        matrix[r][c] = p1[i];
                    }
                }
                if length2 > 0 {
                    for (i, &(r, c)) in coordinates2.iter().enumerate() {
                        matrix[r][c] = p2[i];
                    }
                }
                if length3 > 0 {
                    for (i, &(r, c)) in coordinates3.iter().enumerate() {
                        matrix[r][c] = p3[i];
                    }
                }
                let score = scoring_fn(&matrix);
                consider_record(
                    matrix,
                    score,
                    n_permutations,
                    goal,
                    tolerance,
                    max_records_opt,
                    &mut records,
                    &mut best_score,
                    &mut threshold_score,
                );
                n_permutations += 1;
                if n_permutations % BATCH == 0 {
                    progress_fn(n_permutations, false);
                }
                if sleep_ns != 0 {
                    sleep(Duration::from_nanos(sleep_ns));
                }
                n_permutations < max_permutations
            });
            n_permutations < max_permutations
        });
        n_permutations < max_permutations
    });
    progress_fn(n_permutations, true);
    let records: Vec<[[u8; C]; R]> = records.into_iter().map(|(_, _, m)| m).collect();
    Ok((n_permutations, permutations_truncated, records))
}
