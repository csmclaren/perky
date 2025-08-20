use crate::layouts::{Laterality, LayoutTable, Position};

pub type Effort = f64;

pub type Fingering = (usize, usize, Laterality, Position);

pub type UnigramFingering = (Fingering, Effort);

pub type BigramFingering = (Fingering, Fingering, Effort);

pub type TrigramFingering = (Fingering, Fingering, Fingering, Effort);

#[inline]
fn fast_distance(r1: usize, c1: usize, r2: usize, c2: usize) -> f64 {
    let dx = r2.abs_diff(r1);
    let dy = c2.abs_diff(c1);
    match (dx, dy) {
        (0, 0) => 0.0,
        (0, d) | (d, 0) => d as f64,
        _ => ((dx * dx + dy * dy) as f64).sqrt(),
    }
}

impl<const C: usize, const R: usize> LayoutTable<C, R> {
    pub fn iter_f(&self) -> impl Iterator<Item = UnigramFingering> {
        (0..R).flat_map(move |r| {
            (0..C).filter_map(move |c| {
                self.0[r][c].map(|digit| {
                    let effort = 1.0;
                    ((r, c, digit.0, digit.1), effort)
                })
            })
        })
    }

    pub fn iter_fp(&self) -> impl Iterator<Item = BigramFingering> {
        self.iter_f().flat_map(move |(f1, _effort)| {
            self.iter_f().map(move |(f2, _effort)| {
                let (r1, c1, l1, _p1) = f1;
                let (r2, c2, l2, _p2) = f2;
                let effort = if l1 == l2 {
                    fast_distance(r1, c1, r2, c2)
                } else {
                    1.0
                };
                (f1, f2, effort)
            })
        })
    }

    pub fn iter_ft(&self) -> impl Iterator<Item = TrigramFingering> {
        self.iter_f().flat_map(move |(f1, _)| {
            self.iter_f().flat_map(move |(f2, _)| {
                self.iter_f().map(move |(f3, _)| {
                    let (r1, c1, l1, _p1) = f1;
                    let (r2, c2, l2, _p2) = f2;
                    let (r3, c3, l3, _p3) = f3;
                    let effort = if l1 == l2 {
                        fast_distance(r1, c1, r2, c2)
                    } else {
                        1.0
                    } * if l2 == l3 {
                        fast_distance(r2, c2, r3, c3)
                    } else {
                        1.0
                    };
                    (f1, f2, f3, effort)
                })
            })
        })
    }
}
