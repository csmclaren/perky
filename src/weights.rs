use core::fmt::{self, Display};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Weight {
    Effort,
    Raw,
}

impl Display for Weight {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Weight::Effort => write!(f, "Effort"),
            Weight::Raw => write!(f, "Raw"),
        }
    }
}
