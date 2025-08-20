pub fn calculate_frac(n: u64, t: u64) -> Option<f64> {
    if t == 0 {
        None
    } else {
        Some(n as f64 / t as f64)
    }
}

pub fn calculate_perc(n: u64, t: u64) -> Option<f64> {
    calculate_frac(n, t).map(|f| f * 100.0)
}

pub fn crop_matrix<const C: usize, const R: usize, T>(
    matrix: &[[T; C]; R],
    predicate: impl Fn(&T) -> bool,
) -> (usize, usize, usize, usize) {
    let mut top = 0;
    let mut right = 0;
    let mut bottom = 0;
    let mut left = 0;
    for r in matrix {
        if r.iter().any(|b| predicate(b)) {
            break;
        }
        top += 1;
    }
    for c in (0..C).rev() {
        if matrix.iter().any(|row| predicate(&row[c])) {
            break;
        }
        right += 1;
    }
    for r in matrix.iter().rev() {
        if r.iter().any(|b| predicate(b)) {
            break;
        }
        bottom += 1;
    }
    for c in 0..C {
        if matrix.iter().any(|row| predicate(&row[c])) {
            break;
        }
        left += 1;
    }
    (top, right, bottom, left)
}

pub fn factorial(n: u64) -> u64 {
    (1..=n).product()
}

pub fn index_to_permutation<T: Copy>(mut index: u64, input: &[T]) -> Vec<T> {
    let input_length = input.len();
    debug_assert!(
        index < factorial(input_length as u64),
        "index {} out of bounds for {}-length permutation",
        index,
        input_length
    );
    let mut available = input.to_vec();
    let mut output = Vec::with_capacity(available.len());
    for i in (1..=available.len()).rev() {
        let f = factorial(i as u64 - 1);
        let pos = (index / f) as usize;
        index %= f;
        output.push(available.remove(pos));
    }
    output
}

pub fn index_to_permutation_in_place<const N: usize, T: Copy + Default>(
    mut index: u64,
    input: &[T],
    output: &mut [T],
) {
    let input_length = input.len();
    if input_length == 0 {
        return;
    }
    debug_assert!(
        input_length <= N,
        "input length {} must be <= maximum permutation length {}",
        input_length,
        N
    );
    let output_length = output.len();
    debug_assert!(
        input_length <= output_length,
        "input length {} must be <= output length {}",
        input_length,
        output_length
    );
    let mut f = factorial(input_length as u64);
    debug_assert!(
        index < f,
        "index {} out of bounds for {}-length permutation",
        index,
        input_length
    );
    let mut available = [T::default(); N];
    available[..input_length].copy_from_slice(input);
    f /= input_length as u64;
    let mut remaining = input_length;
    for i in 0..input_length {
        let pos = (index / f) as usize;
        index %= f;
        output[i] = available[pos];
        for j in pos..remaining - 1 {
            available[j] = available[j + 1];
        }
        remaining -= 1;
        if remaining > 1 {
            f /= remaining as u64;
        }
    }
}

pub fn generate_permutations<const N: usize, T>(
    array: [T; N],
    callback: impl FnMut(&[T; N]) -> bool,
) {
    generate_permutations_to_limit(array, N, callback);
}

pub fn generate_permutations_to_limit<const N: usize, T>(
    mut array: [T; N],
    limit: usize,
    mut callback: impl FnMut(&[T; N]) -> bool,
) {
    fn permute<const N: usize, T>(
        slice: &mut [T; N],
        start: usize,
        limit: usize,
        callback: &mut impl FnMut(&[T; N]) -> bool,
    ) -> bool {
        if start == limit {
            return callback(slice);
        }
        for i in start..limit {
            slice.swap(start, i);
            if !permute(slice, start + 1, limit, callback) {
                slice.swap(start, i);
                return false;
            }
            slice.swap(start, i);
        }
        true
    }

    let limit = limit.min(N);
    permute(&mut array, 0, limit, &mut callback);
}
