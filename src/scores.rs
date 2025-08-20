use crate::{
    fingerings::{BigramFingering, TrigramFingering, UnigramFingering},
    ngrams::{BigramKey, BigramTable, TrigramKey, TrigramTable, UnigramKey, UnigramTable},
};

impl UnigramKey {
    #[inline]
    pub fn from_fingering<const C: usize, const R: usize>(
        uf: &UnigramFingering,
        key_table_matrix: &[[u8; C]; R],
    ) -> Self {
        let &((r1, c1, ..), _) = uf;
        Self::from(key_table_matrix[r1][c1])
    }
}

impl BigramKey {
    #[inline]
    pub fn from_fingering<const C: usize, const R: usize>(
        bf: &BigramFingering,
        key_table_matrix: &[[u8; C]; R],
    ) -> Self {
        let &((r1, c1, ..), (r2, c2, ..), _) = bf;
        Self::from((key_table_matrix[r1][c1], key_table_matrix[r2][c2]))
    }
}

impl TrigramKey {
    #[inline]
    pub fn from_fingering<const C: usize, const R: usize>(
        tf: &TrigramFingering,
        key_table_matrix: &[[u8; C]; R],
    ) -> Self {
        let &((r1, c1, ..), (r2, c2, ..), (r3, c3, ..), _) = tf;
        Self::from((
            key_table_matrix[r1][c1],
            key_table_matrix[r2][c2],
            key_table_matrix[r3][c3],
        ))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Score<K> {
    pub key: K,
    pub value: u64,
    pub value_ew: u64,
}

impl<K> Score<K> {
    pub fn is_zero(&self) -> bool {
        self.value | self.value_ew == 0
    }
}

pub enum ScoreMode {
    Detailed,
    SummarySafe,
    SummaryUnsafe,
}

#[inline]
pub fn score_uf<const C: usize, const R: usize>(
    uf: &UnigramFingering,
    key_table_matrix: &[[u8; C]; R],
    unigram_table: &UnigramTable,
) -> Score<UnigramKey> {
    let key = UnigramKey::from_fingering(uf, key_table_matrix);
    let value = unigram_table[key.as_usize()];
    let &(_, effort) = uf;
    let value_ew = (value as f64 * effort) as u64;
    Score {
        key,
        value,
        value_ew,
    }
}

#[inline]
pub fn score_ufs<const C: usize, const R: usize>(
    uf_slice: &[UnigramFingering],
    key_table_matrix: &[[u8; C]; R],
    unigram_table: &UnigramTable,
    mode: ScoreMode,
) -> (Option<Vec<Score<UnigramKey>>>, u64, u64) {
    match mode {
        ScoreMode::Detailed => {
            let (details, sum, sum_ew) =
                score_ufs_with_details(uf_slice, key_table_matrix, unigram_table);
            (Some(details), sum, sum_ew)
        }
        ScoreMode::SummarySafe => {
            let (sum, sum_ew) =
                score_ufs_without_details_safe(uf_slice, key_table_matrix, unigram_table);
            (None, sum, sum_ew)
        }
        ScoreMode::SummaryUnsafe => {
            let (sum, sum_ew) =
                score_ufs_without_details_unsafe(uf_slice, key_table_matrix, unigram_table);
            (None, sum, sum_ew)
        }
    }
}

#[inline]
pub fn score_ufs_with_details<const C: usize, const R: usize>(
    uf_slice: &[UnigramFingering],
    key_table_matrix: &[[u8; C]; R],
    unigram_table: &UnigramTable,
) -> (Vec<Score<UnigramKey>>, u64, u64) {
    uf_slice
        .iter()
        .map(|uf| score_uf(uf, key_table_matrix, unigram_table))
        .fold(
            (Vec::with_capacity(uf_slice.len()), 0, 0),
            |(mut unigrams, a, a_ew), score| {
                unigrams.push(score.clone());
                (unigrams, a + score.value, a_ew + score.value_ew)
            },
        )
}

#[inline]
pub fn score_ufs_without_details_safe<const C: usize, const R: usize>(
    uf_slice: &[UnigramFingering],
    key_table_matrix: &[[u8; C]; R],
    unigram_table: &UnigramTable,
) -> (u64, u64) {
    uf_slice
        .iter()
        .map(|uf| score_uf(uf, key_table_matrix, unigram_table))
        .fold((0, 0), |(a, a_ew), score| {
            (a + score.value, a_ew + score.value_ew)
        })
}

#[inline]
pub fn score_ufs_without_details_unsafe<const C: usize, const R: usize>(
    uf_slice: &[UnigramFingering],
    key_table_matrix: &[[u8; C]; R],
    unigram_table: &UnigramTable,
) -> (u64, u64) {
    let mut a = 0u64;
    let mut a_ew = 0u64;
    for i in 0..uf_slice.len() {
        let ((r, c, ..), effort) = uf_slice[i];
        let b = unsafe { *key_table_matrix.get_unchecked(r).get_unchecked(c) };
        let key = UnigramKey::from(b).as_usize();
        let value = unsafe { *unigram_table.get_unchecked(key) };
        let value_ew = (value as f64 * effort) as u64;
        a += value;
        a_ew += value_ew;
    }
    (a, a_ew)
}

#[inline]
pub fn score_bf<const C: usize, const R: usize>(
    bf: &BigramFingering,
    key_table_matrix: &[[u8; C]; R],
    bigram_table: &BigramTable,
) -> Score<BigramKey> {
    let key = BigramKey::from_fingering(bf, key_table_matrix);
    let value = bigram_table[key.as_usize()];
    let &(.., effort) = bf;
    let value_ew = (value as f64 * effort) as u64;
    Score {
        key,
        value,
        value_ew,
    }
}

#[inline]
pub fn score_bfs<const C: usize, const R: usize>(
    bf_slice: &[BigramFingering],
    key_table_matrix: &[[u8; C]; R],
    bigram_table: &BigramTable,
    mode: ScoreMode,
) -> (Option<Vec<Score<BigramKey>>>, u64, u64) {
    match mode {
        ScoreMode::Detailed => {
            let (details, sum, sum_ew) =
                score_bfs_with_details(bf_slice, key_table_matrix, bigram_table);
            (Some(details), sum, sum_ew)
        }
        ScoreMode::SummarySafe => {
            let (sum, sum_ew) =
                score_bfs_without_details_safe(bf_slice, key_table_matrix, bigram_table);
            (None, sum, sum_ew)
        }
        ScoreMode::SummaryUnsafe => {
            let (sum, sum_ew) =
                score_bfs_without_details_unsafe(bf_slice, key_table_matrix, bigram_table);
            (None, sum, sum_ew)
        }
    }
}

#[inline]
pub fn score_bfs_with_details<const C: usize, const R: usize>(
    bf_slice: &[BigramFingering],
    key_table_matrix: &[[u8; C]; R],
    bigram_table: &BigramTable,
) -> (Vec<Score<BigramKey>>, u64, u64) {
    bf_slice
        .iter()
        .map(|bf| score_bf(bf, key_table_matrix, bigram_table))
        .fold(
            (Vec::with_capacity(bf_slice.len()), 0, 0),
            |(mut bigrams, a, a_ew), score| {
                bigrams.push(score.clone());
                (bigrams, a + score.value, a_ew + score.value_ew)
            },
        )
}

#[inline]
pub fn score_bfs_without_details_safe<const C: usize, const R: usize>(
    bf_slice: &[BigramFingering],
    key_table_matrix: &[[u8; C]; R],
    bigram_table: &BigramTable,
) -> (u64, u64) {
    bf_slice
        .iter()
        .map(|bf| score_bf(bf, key_table_matrix, bigram_table))
        .fold((0, 0), |(a, a_ew), score| {
            (a + score.value, a_ew + score.value_ew)
        })
}

#[inline]
pub fn score_bfs_without_details_unsafe<const C: usize, const R: usize>(
    bf_slice: &[BigramFingering],
    key_table_matrix: &[[u8; C]; R],
    bigram_table: &BigramTable,
) -> (u64, u64) {
    let mut a = 0u64;
    let mut a_ew = 0u64;
    for i in 0..bf_slice.len() {
        let ((r1, c1, ..), (r2, c2, ..), effort) = bf_slice[i];
        let b1 = unsafe { *key_table_matrix.get_unchecked(r1).get_unchecked(c1) };
        let b2 = unsafe { *key_table_matrix.get_unchecked(r2).get_unchecked(c2) };
        let key = BigramKey::from((b1, b2)).as_usize();
        let value = unsafe { *bigram_table.get_unchecked(key) };
        let value_ew = (value as f64 * effort) as u64;
        a += value;
        a_ew += value_ew;
    }
    (a, a_ew)
}

#[inline]
pub fn score_tf<const C: usize, const R: usize>(
    tf: &TrigramFingering,
    key_table_matrix: &[[u8; C]; R],
    trigram_table: &TrigramTable,
) -> Score<TrigramKey> {
    let key = TrigramKey::from_fingering(tf, key_table_matrix);
    let value = trigram_table[key.as_usize()];
    let &(.., effort) = tf;
    let value_ew = (value as f64 * effort) as u64;
    Score {
        key,
        value,
        value_ew,
    }
}

#[inline]
pub fn score_tfs<const C: usize, const R: usize>(
    tf_slice: &[TrigramFingering],
    key_table_matrix: &[[u8; C]; R],
    trigram_table: &TrigramTable,
    mode: ScoreMode,
) -> (Option<Vec<Score<TrigramKey>>>, u64, u64) {
    match mode {
        ScoreMode::Detailed => {
            let (details, sum, sum_ew) =
                score_tfs_with_details(tf_slice, key_table_matrix, trigram_table);
            (Some(details), sum, sum_ew)
        }
        ScoreMode::SummarySafe => {
            let (sum, sum_ew) =
                score_tfs_without_details_safe(tf_slice, key_table_matrix, trigram_table);
            (None, sum, sum_ew)
        }
        ScoreMode::SummaryUnsafe => {
            let (sum, sum_ew) =
                score_tfs_without_details_unsafe(tf_slice, key_table_matrix, trigram_table);
            (None, sum, sum_ew)
        }
    }
}

#[inline]
pub fn score_tfs_with_details<const C: usize, const R: usize>(
    tf_slice: &[TrigramFingering],
    key_table_matrix: &[[u8; C]; R],
    trigram_table: &TrigramTable,
) -> (Vec<Score<TrigramKey>>, u64, u64) {
    tf_slice
        .iter()
        .map(|tf| score_tf(tf, key_table_matrix, trigram_table))
        .fold(
            (Vec::with_capacity(tf_slice.len()), 0, 0),
            |(mut trigrams, a, a_ew), score| {
                trigrams.push(score.clone());
                (trigrams, a + score.value, a_ew + score.value_ew)
            },
        )
}

#[inline]
pub fn score_tfs_without_details_safe<const C: usize, const R: usize>(
    tf_slice: &[TrigramFingering],
    key_table_matrix: &[[u8; C]; R],
    trigram_table: &TrigramTable,
) -> (u64, u64) {
    tf_slice
        .iter()
        .map(|tf| score_tf(tf, key_table_matrix, trigram_table))
        .fold((0, 0), |(a, a_ew), score| {
            (a + score.value, a_ew + score.value_ew)
        })
}

#[inline]
pub fn score_tfs_without_details_unsafe<const C: usize, const R: usize>(
    tf_slice: &[TrigramFingering],
    key_table_matrix: &[[u8; C]; R],
    trigram_table: &TrigramTable,
) -> (u64, u64) {
    let mut a = 0u64;
    let mut a_ew = 0u64;
    for i in 0..tf_slice.len() {
        let ((r1, c1, ..), (r2, c2, ..), (r3, c3, ..), effort) = tf_slice[i];
        let b1 = unsafe { *key_table_matrix.get_unchecked(r1).get_unchecked(c1) };
        let b2 = unsafe { *key_table_matrix.get_unchecked(r2).get_unchecked(c2) };
        let b3 = unsafe { *key_table_matrix.get_unchecked(r3).get_unchecked(c3) };
        let key = TrigramKey::from((b1, b2, b3)).as_usize();
        let value = unsafe { *trigram_table.get_unchecked(key) };
        let value_ew = (value as f64 * effort) as u64;
        a += value;
        a_ew += value_ew;
    }
    (a, a_ew)
}
