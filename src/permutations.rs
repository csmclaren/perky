use core::{cmp, error::Error, sync::atomic, time::Duration};

use std::{
    sync::{Arc, Mutex},
    thread::sleep,
};

use rayon::iter::{IntoParallelIterator, ParallelIterator};

use crate::{
    goals::Goal,
    util::math::{factorial, generate_permutations_to_limit, index_to_permutation_in_place},
};

pub fn convert_option_vec_to_array<const N: usize, T: Default + Copy>(
    opt_vec: Option<Vec<T>>,
) -> Result<([T; N], usize), Box<dyn Error>> {
    let mut array = [T::default(); N];
    match opt_vec {
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
    max_permutations: u64,
    parallel: bool,
    sleep_ns: u64,
    opt_truncate: Option<u64>,
) -> Result<(Vec<[[u8; C]; R]>, u64, u64), Box<dyn Error>> {
    if parallel {
        permute_and_substitute_parallel(
            &matrix,
            region1,
            region2,
            region3,
            progress_fn,
            scoring_fn,
            goal,
            tolerance,
            max_permutations,
            sleep_ns,
            opt_truncate,
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
            max_permutations,
            sleep_ns,
            opt_truncate,
        )
    }
}

pub fn permute_and_substitute_parallel<const C: usize, const R: usize, const N: usize>(
    matrix: &[[u8; C]; R],
    region1: ([u8; N], usize, &[(usize, usize)]),
    region2: ([u8; N], usize, &[(usize, usize)]),
    region3: ([u8; N], usize, &[(usize, usize)]),
    progress_fn: impl FnMut(u64, bool) -> bool + Send + Sync,
    scoring_fn: impl Fn(&[[u8; C]; R]) -> u64 + Sync,
    goal: Goal,
    #[allow(unused_variables)] tolerance: f64,
    max_permutations: u64,
    sleep_ns: u64,
    opt_truncate: Option<u64>,
) -> Result<(Vec<[[u8; C]; R]>, u64, u64), Box<dyn Error>> {
    const BATCH: u64 = 1000;
    use Goal::*;
    let initial_score = match goal {
        Max => 0,
        Min => u64::MAX,
    };
    if max_permutations == 0 {
        return Ok((Vec::new(), initial_score, 0));
    }
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
    let total = total1
        .saturating_mul(total2)
        .saturating_mul(total3)
        .min(max_permutations);
    let count = Arc::new(atomic::AtomicU64::new(0));
    let progress_fn = Arc::new(Mutex::new(progress_fn));
    let (best_matrices, best_score) = (0..total)
        .into_par_iter()
        .fold(
            || (Vec::new(), initial_score, 0u64),
            |(mut local_best_matrices, mut local_best_score, mut local_n_permutations), index| {
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
                let is_better = match goal {
                    Max => score > local_best_score,
                    Min => score < local_best_score,
                };
                let is_equal = score == local_best_score;
                if is_better {
                    local_best_score = score;
                    local_best_matrices.clear();
                    if opt_truncate.map_or(true, |truncate| {
                        (local_best_matrices.len() as u64) < truncate
                    }) {
                        local_best_matrices.push(matrix);
                    }
                } else if is_equal {
                    if opt_truncate.map_or(true, |truncate| {
                        (local_best_matrices.len() as u64) < truncate
                    }) {
                        local_best_matrices.push(matrix);
                    }
                }
                local_n_permutations += 1;
                if local_n_permutations % BATCH == 0 {
                    let current = count.fetch_add(BATCH, atomic::Ordering::Relaxed) + BATCH;
                    if let Ok(mut progress_fn) = progress_fn.lock() {
                        progress_fn(current, false);
                    }
                    if sleep_ns != 0 {
                        sleep(Duration::from_nanos(sleep_ns));
                    }
                }
                (local_best_matrices, local_best_score, local_n_permutations)
            },
        )
        .map(
            |(local_best_matrices, local_best_score, local_n_permutations)| {
                let remaining = local_n_permutations % BATCH;
                if remaining != 0 {
                    count.fetch_add(remaining, atomic::Ordering::Relaxed);
                }
                (local_best_matrices, local_best_score)
            },
        )
        .reduce(
            || (Vec::new(), initial_score),
            |(mut local_best_matrices1, local_best_score1),
             (local_best_matrices2, local_best_score2)| match goal {
                Max => match local_best_score1.cmp(&local_best_score2) {
                    cmp::Ordering::Greater => (local_best_matrices1, local_best_score1),
                    cmp::Ordering::Less => (local_best_matrices2, local_best_score2),
                    cmp::Ordering::Equal => {
                        if let Some(truncate) = opt_truncate {
                            let remaining =
                                truncate.saturating_sub(local_best_matrices1.len() as u64) as usize;
                            local_best_matrices1
                                .extend(local_best_matrices2.into_iter().take(remaining));
                        } else {
                            local_best_matrices1.extend(local_best_matrices2);
                        }
                        (local_best_matrices1, local_best_score1)
                    }
                },
                Min => match local_best_score1.cmp(&local_best_score2) {
                    cmp::Ordering::Less => (local_best_matrices1, local_best_score1),
                    cmp::Ordering::Greater => (local_best_matrices2, local_best_score2),
                    cmp::Ordering::Equal => {
                        if let Some(truncate) = opt_truncate {
                            let remaining =
                                truncate.saturating_sub(local_best_matrices1.len() as u64) as usize;
                            local_best_matrices1
                                .extend(local_best_matrices2.into_iter().take(remaining));
                        } else {
                            local_best_matrices1.extend(local_best_matrices2);
                        }
                        (local_best_matrices1, local_best_score1)
                    }
                },
            },
        );
    let total_count = count.load(atomic::Ordering::Relaxed);
    if let Ok(mut progress_fn) = progress_fn.lock() {
        progress_fn(total_count, true);
    }
    Ok((best_matrices, best_score, total_count))
}

pub fn permute_and_substitute_sequential<const C: usize, const R: usize, const N: usize>(
    matrix: &[[u8; C]; R],
    region1: ([u8; N], usize, &[(usize, usize)]),
    region2: ([u8; N], usize, &[(usize, usize)]),
    region3: ([u8; N], usize, &[(usize, usize)]),
    mut progress_fn: impl FnMut(u64, bool) -> bool,
    scoring_fn: impl Fn(&[[u8; C]; R]) -> u64,
    goal: Goal,
    #[allow(unused_variables)] tolerance: f64,
    max_permutations: u64,
    sleep_ns: u64,
    opt_truncate: Option<u64>,
) -> Result<(Vec<[[u8; C]; R]>, u64, u64), Box<dyn Error>> {
    const BATCH: u64 = 1000000;
    use Goal::*;
    let initial_score = match goal {
        Max => 0,
        Min => u64::MAX,
    };
    if max_permutations == 0 {
        return Ok((Vec::new(), initial_score, 0));
    }
    let (array1, length1, coordinates1) = region1;
    let (array2, length2, coordinates2) = region2;
    let (array3, length3, coordinates3) = region3;
    let length1 = length1.min(N);
    let length2 = length2.min(N);
    let length3 = length3.min(N);
    let coordinates1 = &coordinates1[..length1];
    let coordinates2 = &coordinates2[..length2];
    let coordinates3 = &coordinates3[..length3];
    let mut best_matrices = Vec::new();
    let mut best_score = initial_score;
    let mut n_permutations = 0u64;
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
                n_permutations += 1;
                if n_permutations % BATCH == 0 {
                    progress_fn(n_permutations, false);
                }
                let score = scoring_fn(&matrix);
                let is_better = match goal {
                    Max => score > best_score,
                    Min => score < best_score,
                };
                let is_equal = score == best_score;
                if is_better {
                    best_score = score;
                    best_matrices.clear();
                    if opt_truncate.map_or(true, |t| (best_matrices.len() as u64) < t) {
                        best_matrices.push(matrix);
                    }
                } else if is_equal {
                    if opt_truncate.map_or(true, |t| (best_matrices.len() as u64) < t) {
                        best_matrices.push(matrix);
                    }
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
    Ok((best_matrices, best_score, n_permutations))
}
