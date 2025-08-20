use core::error::Error;

use std::{fs::File, path::Path};

use serde_json::Value;

use crate::{json::read_enveloped_data, tables::Table};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Key {
    Byte(u8),
    One,
    Two,
    Three,
}

impl From<Key> for Value {
    fn from(value: Key) -> Self {
        use Key::*;
        use Value::*;
        match value {
            Byte(b) => String((b as char).to_string()),
            One => Number(1.into()),
            Two => Number(2.into()),
            Three => Number(3.into()),
        }
    }
}

impl TryFrom<&Value> for Key {
    type Error = String;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        use Key::*;
        use Value::*;
        Ok(match value {
            Number(n) => match n.as_u64() {
                Some(1) => One,
                Some(2) => Two,
                Some(3) => Three,
                Some(n) => Err(format!("Invalid key number '{}': expected 1, 2, or 3", n))?,
                _ => Err("Invalid key number: expected 1, 2, or 3")?,
            },
            String(s) if s.len() != 1 || !s.is_ascii() => Err(format!(
                "Invalid key string '{}': \
                 expected a single ASCII character",
                s
            ))?,
            String(s) if s.chars().any(|ch| ('\x01'..='\x03').contains(&ch)) => Err(format!(
                "Invalid key string '{}': \
                 expected a single ASCII character, and the control characters \
                 SOH, STX, and ETX are reserved.",
                s
            ))?,
            String(s) => Byte(s.as_bytes()[0]),
            _ => Err("Invalid type: expected 1, 2, 3, or a string of a single ASCII character")?,
        })
    }
}

pub struct KeyTable<const C: usize, const R: usize>(pub Table<C, R, Key>);

impl<const C: usize, const R: usize> KeyTable<C, R> {
    pub fn from_byte_matrix(slice: &[[u8; C]; R]) -> Self {
        use Key::*;
        let mut key_table = Self::default();
        for (r, row) in slice.iter().enumerate() {
            for (c, byte) in row.iter().enumerate() {
                key_table.0[r][c] = match byte {
                    0 => None,
                    1 => Some(One),
                    2 => Some(Two),
                    3 => Some(Three),
                    b => Some(Byte(*b)),
                };
            }
        }
        key_table
    }

    pub fn to_byte_matrix(&self) -> [[u8; C]; R] {
        use Key::*;
        let mut byte_matrix = [[0u8; C]; R];
        for (r, row) in self.0.iter().enumerate() {
            for (c, cell) in row.iter().enumerate() {
                byte_matrix[r][c] = match cell {
                    None => 0,
                    Some(Byte(b)) => *b,
                    Some(One) => 1,
                    Some(Two) => 2,
                    Some(Three) => 3,
                };
            }
        }
        byte_matrix
    }

    pub fn read_from_path(path: &Path) -> Result<Self, Box<dyn Error>> {
        const EXPECTED_VERSION: u64 = 1;
        let file = File::open(path)?;
        let value = read_enveloped_data::<_, Value>(file, EXPECTED_VERSION)?;
        Ok(KeyTable::try_from(&value)?)
    }
}

impl<const C: usize, const R: usize> Default for KeyTable<C, R> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<const C: usize, const R: usize> From<&KeyTable<C, R>> for Value {
    fn from(value: &KeyTable<C, R>) -> Self {
        Value::from(&value.0)
    }
}

impl<const C: usize, const R: usize> TryFrom<&Value> for KeyTable<C, R> {
    type Error = String;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        Table::<C, R, Key>::try_from(value).map(KeyTable)
    }
}
