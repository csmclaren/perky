use core::fmt::{self, Display};

use std::{collections::HashSet, io, sync::LazyLock};

use strum::{EnumCount, VariantNames};

use strum_macros::{EnumCount, VariantNames};

use termcolor::{ColorSpec, WriteColor};

use crate::{
    fingerings::{BigramFingering, TrigramFingering, UnigramFingering},
    goals::Goal,
    layouts::{Laterality, LayoutTable, Position},
    ui::styles::WriteStyled,
};

pub fn filter_lt(f: &UnigramFingering) -> bool {
    let ((.., l, p), _) = *f;
    l == Laterality::Left && p == Position::Thumb
}

pub fn filter_li(f: &UnigramFingering) -> bool {
    let ((.., l, p), _) = *f;
    l == Laterality::Left && p == Position::Index
}

pub fn filter_lm(f: &UnigramFingering) -> bool {
    let ((.., l, p), _) = *f;
    l == Laterality::Left && p == Position::Middle
}

pub fn filter_lr(f: &UnigramFingering) -> bool {
    let ((.., l, p), _) = *f;
    l == Laterality::Left && p == Position::Ring
}

pub fn filter_lp(f: &UnigramFingering) -> bool {
    let ((.., l, p), _) = *f;
    l == Laterality::Left && p == Position::Pinky
}

pub fn filter_lh(f: &UnigramFingering) -> bool {
    let ((.., l, _), _) = *f;
    l == Laterality::Left
}

pub fn filter_rt(f: &UnigramFingering) -> bool {
    let ((.., l, p), _) = *f;
    l == Laterality::Right && p == Position::Thumb
}

pub fn filter_ri(f: &UnigramFingering) -> bool {
    let ((.., l, p), _) = *f;
    l == Laterality::Right && p == Position::Index
}

pub fn filter_rm(f: &UnigramFingering) -> bool {
    let ((.., l, p), _) = *f;
    l == Laterality::Right && p == Position::Middle
}

pub fn filter_rr(f: &UnigramFingering) -> bool {
    let ((.., l, p), _) = *f;
    l == Laterality::Right && p == Position::Ring
}

pub fn filter_rp(f: &UnigramFingering) -> bool {
    let ((.., l, p), _) = *f;
    l == Laterality::Right && p == Position::Pinky
}

pub fn filter_rh(f: &UnigramFingering) -> bool {
    let ((.., l, _), _) = *f;
    l == Laterality::Right
}

pub fn filter_distinct_pairs(fp: &BigramFingering) -> bool {
    let ((r1, c1, ..), (r2, c2, ..), _) = *fp;
    (r1, c1) != (r2, c2)
}

pub fn filter_hsb(fp: &BigramFingering) -> bool {
    use Position::*;
    let ((r1, c1, l1, p1), (r2, c2, l2, p2), _) = *fp;
    l1 == l2
        && c1.abs_diff(c2) >= 1
        && r1.abs_diff(r2) == 1
        && ((matches!(p1, Middle | Ring) && r1 > r2) || (matches!(p2, Middle | Ring) && r2 > r1))
}

pub fn filter_fsb(fp: &BigramFingering) -> bool {
    use Position::*;
    let ((r1, c1, l1, p1), (r2, c2, l2, p2), _) = *fp;
    l1 == l2
        && c1.abs_diff(c2) >= 1
        && r1.abs_diff(r2) > 1
        && ((matches!(p1, Middle | Ring) && r1 > r2) || (matches!(p2, Middle | Ring) && r2 > r1))
}

pub fn filter_lsb(fp: &BigramFingering) -> bool {
    use Position::*;
    let ((_, c1, l1, p1), (_, c2, l2, p2), _) = *fp;
    l1 == l2 && c1.abs_diff(c2) >= 2 && matches!((p1, p2), (Middle, Index) | (Index, Middle))
}

pub fn filter_irb(fp: &BigramFingering) -> bool {
    use Position::*;
    let ((r1, c1, l1, p1), (r2, c2, l2, p2), _) = *fp;
    l1 == l2
        && r1 == r2
        && c1.abs_diff(c2) == 1
        && matches!((p1, p2), (Pinky, Ring) | (Ring, Middle) | (Middle, Index))
}

pub fn filter_orb(fp: &BigramFingering) -> bool {
    use Position::*;
    let ((r1, c1, l1, p1), (r2, c2, l2, p2), _) = *fp;
    l1 == l2
        && r1 == r2
        && c1.abs_diff(c2) == 1
        && matches!((p1, p2), (Index, Middle) | (Middle, Ring) | (Ring, Pinky))
}

pub fn filter_sfb(fp: &BigramFingering) -> bool {
    let ((.., l1, p1), (.., l2, p2), _) = *fp;
    l1 == l2 && p1 == p2
}

pub fn filter_distinct_triples(ft: &TrigramFingering) -> bool {
    let ((r1, c1, ..), (r2, c2, ..), (r3, c3, ..), _) = *ft;
    let p1 = (r1, c1);
    let p2 = (r2, c2);
    let p3 = (r3, c3);
    p1 != p2 && p1 != p3 && p2 != p3
}

pub fn filter_alt(ft: &TrigramFingering) -> bool {
    let ((_, _, l1, _), (_, _, l2, _), (_, _, l3, _), _) = *ft;
    l1 == l3 && l2 != l1
}

pub fn filter_one(ft: &TrigramFingering) -> bool {
    let ((_, c1, l1, p1), (_, c2, l2, p2), (_, c3, l3, p3), _) = *ft;
    l1 == l2
        && l2 == l3
        && p1 != p2
        && p2 != p3
        && p1 != p3
        && ((c1 < c2 && c2 < c3) || (c1 > c2 && c2 > c3))
}

pub fn filter_red(ft: &TrigramFingering) -> bool {
    let ((_, c1, l1, p1), (_, c2, l2, p2), (_, c3, l3, p3), _) = *ft;
    l1 == l2
        && l2 == l3
        && p1 != p2
        && p2 != p3
        && p1 != p3
        && ((c1 < c2 && c2 > c3) || (c1 > c2 && c2 < c3))
}

pub fn filter_rol(ft: &TrigramFingering) -> bool {
    let ((_, _, l1, p1), (_, _, l2, p2), (_, _, l3, p3), _) = *ft;
    (l1 == l2 && l1 != l3 && p1 != p2) || (l2 == l3 && l2 != l1 && p2 != p3)
}

pub static STYLE_UNIGRAM_METRIC: LazyLock<ColorSpec> = LazyLock::new(|| {
    let mut color_spec = ColorSpec::new();
    color_spec.set_bold(true);
    color_spec
});

pub static STYLE_BIGRAM_METRIC: LazyLock<ColorSpec> = LazyLock::new(|| {
    let mut color_spec = ColorSpec::new();
    color_spec.set_bold(true);
    color_spec
});

pub static STYLE_TRIGRAM_METRIC: LazyLock<ColorSpec> = LazyLock::new(|| {
    let mut color_spec = ColorSpec::new();
    color_spec.set_bold(true);
    color_spec
});

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, EnumCount, VariantNames)]
#[repr(usize)]
pub enum UnigramMetric {
    Lt,
    Li,
    Lm,
    Lr,
    Lp,
    Lh,
    Rt,
    Ri,
    Rm,
    Rr,
    Rp,
    Rh,
}

impl UnigramMetric {
    pub const VARIANT_ARRAY: [Self; Self::COUNT] = [
        Self::Lt,
        Self::Li,
        Self::Lm,
        Self::Lr,
        Self::Lp,
        Self::Lh,
        Self::Rt,
        Self::Ri,
        Self::Rm,
        Self::Rr,
        Self::Rp,
        Self::Rh,
    ];

    pub fn as_usize(self) -> usize {
        self as usize
    }

    pub fn filter_fn(self) -> fn(&UnigramFingering) -> bool {
        use UnigramMetric::*;
        match self {
            Lt => filter_lt,
            Li => filter_li,
            Lm => filter_lm,
            Lr => filter_lr,
            Lp => filter_lp,
            Lh => filter_lh,
            Rt => filter_rt,
            Ri => filter_ri,
            Rm => filter_rm,
            Rr => filter_rr,
            Rp => filter_rp,
            Rh => filter_rh,
        }
    }

    pub fn goal(self) -> Goal {
        use Goal::*;
        use UnigramMetric::*;
        match self {
            Lt | Li | Lm | Lh | Rt | Ri | Rm | Rh => Max,
            Lr | Lp | Rr | Rp => Min,
        }
    }
}

impl Display for UnigramMetric {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl WriteStyled for UnigramMetric {
    fn write_styled(&self, writer: &mut dyn WriteColor) -> io::Result<()> {
        writer.set_color(&STYLE_UNIGRAM_METRIC)?;
        write!(writer, "{}", self.to_string())?;
        writer.reset()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, EnumCount, VariantNames)]
#[repr(usize)]
pub enum BigramMetric {
    Fsb,
    Hsb,
    Irb,
    Lsb,
    Orb,
    Sfb,
}

impl BigramMetric {
    pub const VARIANT_ARRAY: [Self; Self::COUNT] = [
        Self::Fsb,
        Self::Hsb,
        Self::Irb,
        Self::Lsb,
        Self::Orb,
        Self::Sfb,
    ];

    pub fn as_usize(self) -> usize {
        self as usize
    }

    pub fn filter_fn(self) -> fn(&BigramFingering) -> bool {
        use BigramMetric::*;
        match self {
            Fsb => filter_fsb,
            Hsb => filter_hsb,
            Irb => filter_irb,
            Lsb => filter_lsb,
            Orb => filter_orb,
            Sfb => filter_sfb,
        }
    }

    pub fn goal(self) -> Goal {
        use BigramMetric::*;
        use Goal::*;
        match self {
            Irb | Orb => Max,
            Fsb | Hsb | Lsb | Sfb => Min,
        }
    }
}

impl Display for BigramMetric {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl WriteStyled for BigramMetric {
    fn write_styled(&self, writer: &mut dyn WriteColor) -> io::Result<()> {
        writer.set_color(&STYLE_BIGRAM_METRIC)?;
        write!(writer, "{}", self.to_string())?;
        writer.reset()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, EnumCount, VariantNames)]
#[repr(usize)]
pub enum TrigramMetric {
    Alt,
    One,
    Red,
    Rol,
}

impl TrigramMetric {
    pub const VARIANT_ARRAY: [Self; Self::COUNT] = [Self::Alt, Self::One, Self::Red, Self::Rol];

    pub fn as_usize(self) -> usize {
        self as usize
    }

    pub fn filter_fn(self) -> fn(&TrigramFingering) -> bool {
        use TrigramMetric::*;
        match self {
            Alt => filter_alt,
            One => filter_one,
            Red => filter_red,
            Rol => filter_rol,
        }
    }

    pub fn goal(self) -> Goal {
        use Goal::*;
        use TrigramMetric::*;
        match self {
            Alt | One | Red | Rol => Min,
        }
    }
}

impl Display for TrigramMetric {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl WriteStyled for TrigramMetric {
    fn write_styled(&self, writer: &mut dyn WriteColor) -> io::Result<()> {
        writer.set_color(&STYLE_TRIGRAM_METRIC)?;
        write!(writer, "{}", self.to_string())?;
        writer.reset()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, VariantNames)]
pub enum Metric {
    Unigram(UnigramMetric),
    Bigram(BigramMetric),
    Trigram(TrigramMetric),
}

static VARIANTS: LazyLock<Vec<&'static str>> = LazyLock::new(|| {
    [
        UnigramMetric::VARIANTS,
        BigramMetric::VARIANTS,
        TrigramMetric::VARIANTS,
    ]
    .concat()
});

impl Metric {
    pub fn get_variables() -> HashSet<String> {
        VARIANTS.iter().map(|&s| s.to_lowercase()).collect()
    }
}

impl Display for Metric {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Metric::*;
        match self {
            Unigram(metric) => metric.fmt(f),
            Bigram(metric) => metric.fmt(f),
            Trigram(metric) => metric.fmt(f),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, VariantNames)]
pub enum SortDirection {
    Ascending,
    Descending,
}

impl Display for SortDirection {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct SortRule {
    pub metric: Metric,
    pub sort_direction: SortDirection,
}

impl Display for SortRule {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {}", self.metric, self.sort_direction)
    }
}

pub fn partition_sort_rules(
    sort_rules: &[SortRule],
) -> (Vec<UnigramMetric>, Vec<BigramMetric>, Vec<TrigramMetric>) {
    let mut unigram_metrics = Vec::new();
    let mut bigram_metrics = Vec::new();
    let mut trigram_metrics = Vec::new();
    for sort_rule in sort_rules {
        use Metric::*;
        match sort_rule.metric {
            Unigram(unigram_metric) => unigram_metrics.push(unigram_metric),
            Bigram(bigram_metric) => bigram_metrics.push(bigram_metric),
            Trigram(trigram_metric) => trigram_metrics.push(trigram_metric),
        };
    }
    (unigram_metrics, bigram_metrics, trigram_metrics)
}

pub struct UnigramFingerings<const C: usize, const R: usize>(
    Vec<UnigramFingering>,
    [Vec<UnigramFingering>; UnigramMetric::COUNT],
);

impl<const C: usize, const R: usize> UnigramFingerings<C, R> {
    pub fn get(&self) -> &Vec<UnigramFingering> {
        &self.0
    }

    pub fn get_by_metric(&self, metric: UnigramMetric) -> &Vec<UnigramFingering> {
        &self.1[metric.as_usize()]
    }
}

pub struct BigramFingerings<const C: usize, const R: usize>(
    Vec<BigramFingering>,
    [Vec<BigramFingering>; BigramMetric::COUNT],
);

impl<const C: usize, const R: usize> BigramFingerings<C, R> {
    pub fn get(&self) -> &Vec<BigramFingering> {
        &self.0
    }

    pub fn get_by_metric(&self, metric: BigramMetric) -> &Vec<BigramFingering> {
        &self.1[metric.as_usize()]
    }
}

pub struct TrigramFingerings<const C: usize, const R: usize>(
    Vec<TrigramFingering>,
    [Vec<TrigramFingering>; TrigramMetric::COUNT],
);

impl<const C: usize, const R: usize> TrigramFingerings<C, R> {
    pub fn get(&self) -> &Vec<TrigramFingering> {
        &self.0
    }

    pub fn get_by_metric(&self, metric: TrigramMetric) -> &Vec<TrigramFingering> {
        &self.1[metric.as_usize()]
    }
}

impl<const C: usize, const R: usize> LayoutTable<C, R> {
    pub fn unigram_fingerings(&self) -> UnigramFingerings<C, R> {
        let fs = self.iter_f().collect::<Vec<_>>();
        let fs_by_metric = UnigramMetric::VARIANT_ARRAY.map(|metric| {
            fs.iter()
                .cloned()
                .filter(|f| metric.filter_fn()(f))
                .collect()
        });
        UnigramFingerings(fs, fs_by_metric)
    }

    pub fn bigram_fingerings(&self) -> BigramFingerings<C, R> {
        let fs = self
            .iter_fp()
            .filter(filter_distinct_pairs)
            .collect::<Vec<_>>();
        let fs_by_metric = BigramMetric::VARIANT_ARRAY.map(|metric| {
            fs.iter()
                .cloned()
                .filter(|f| metric.filter_fn()(f))
                .collect()
        });
        BigramFingerings(fs, fs_by_metric)
    }

    pub fn trigram_fingerings(&self) -> TrigramFingerings<C, R> {
        let fs = self
            .iter_ft()
            .filter(filter_distinct_triples)
            .collect::<Vec<_>>();
        let fs_by_metric = TrigramMetric::VARIANT_ARRAY.map(|metric| {
            fs.iter()
                .cloned()
                .filter(|f| metric.filter_fn()(f))
                .collect()
        });
        TrigramFingerings(fs, fs_by_metric)
    }
}
