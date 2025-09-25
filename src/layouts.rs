use core::{
    error::Error,
    fmt::{self, Display},
};

use std::{fs::File, path::Path};

use serde_json::Value;

use crate::{json::read_enveloped_data, tables::Table};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Laterality {
    Left,
    Right,
}

impl Laterality {
    pub fn as_char(&self) -> char {
        use Laterality::*;
        match self {
            Left => 'l',
            Right => 'r',
        }
    }
}

impl Display for Laterality {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_char())
    }
}

impl TryFrom<char> for Laterality {
    type Error = String;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        use Laterality::*;
        match value {
            'l' => Ok(Left),
            'r' => Ok(Right),
            _ => Err(format!("Invalid laterality character '{}'", value)),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Position {
    Thumb,
    Index,
    Middle,
    Ring,
    Pinky,
}

impl Position {
    pub fn as_char(&self) -> char {
        use Position::*;
        match self {
            Thumb => 't',
            Index => 'i',
            Middle => 'm',
            Ring => 'r',
            Pinky => 'p',
        }
    }
}

impl Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_char())
    }
}

impl TryFrom<char> for Position {
    type Error = String;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        use Position::*;
        match value {
            't' => Ok(Thumb),
            'i' => Ok(Index),
            'm' => Ok(Middle),
            'r' => Ok(Ring),
            'p' => Ok(Pinky),
            _ => Err(format!("Invalid position character '{}'", value)),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Digit(pub Laterality, pub Position);

impl Display for Digit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}", self.0, self.1)
    }
}

impl From<Digit> for Value {
    fn from(value: Digit) -> Value {
        Value::String(value.to_string())
    }
}

impl TryFrom<&str> for Digit {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut chars = value.chars();
        match (chars.next(), chars.next(), chars.next()) {
            (Some(c1), Some(c2), None) => {
                let laterality = Laterality::try_from(c1)?;
                let position = Position::try_from(c2)?;
                Ok(Digit(laterality, position))
            }
            _ => Err(format!(
                "Invalid digit string '{}': expected two ASCII characters",
                value
            )),
        }
    }
}

impl TryFrom<&Value> for Digit {
    type Error = String;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        match value {
            Value::String(s) => Digit::try_from(s.as_str()),
            _ => Err("Invalid type: expected a string of two ASCII characters".into()),
        }
    }
}

pub struct LayoutTable<const C: usize, const R: usize>(pub Table<C, R, Digit>);

impl<const C: usize, const R: usize> LayoutTable<C, R> {
    pub fn mask<F: FnMut(usize, usize, &Digit) -> bool>(&mut self, mut predicate: F) {
        (0..R).for_each(|r| {
            (0..C).for_each(|c| {
                if let Some(digit) = &self.0[r][c] {
                    if !predicate(r, c, digit) {
                        self.0[r][c] = None;
                    }
                }
            })
        })
    }

    pub fn read_from_path(path: &Path) -> Result<Self, Box<dyn Error>> {
        const EXPECTED_VERSION: u64 = 1;
        let file = File::open(path)?;
        let value = read_enveloped_data::<_, Value>(file, EXPECTED_VERSION)?;
        Ok(LayoutTable::try_from(&value)?)
    }
}

impl<const C: usize, const R: usize> Default for LayoutTable<C, R> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<const C: usize, const R: usize> From<&LayoutTable<C, R>> for Value {
    fn from(value: &LayoutTable<C, R>) -> Self {
        Value::from(&value.0)
    }
}

impl<const C: usize, const R: usize> TryFrom<&Value> for LayoutTable<C, R> {
    type Error = String;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        Table::<C, R, Digit>::try_from(value).map(LayoutTable)
    }
}
