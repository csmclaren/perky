use core::{
    array,
    convert::TryFrom,
    ops::{Deref, DerefMut},
};

use serde_json::Value;

pub struct Table<const C: usize, const R: usize, T>(pub [[Option<T>; C]; R]);

impl<const C: usize, const R: usize, T> Deref for Table<C, R, T> {
    type Target = [[Option<T>; C]; R];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<const C: usize, const R: usize, T> DerefMut for Table<C, R, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<const C: usize, const R: usize, T: Copy> From<&Table<C, R, T>> for Value
where
    Value: From<T>,
{
    fn from(value: &Table<C, R, T>) -> Self {
        use Value::*;
        let mut rows = value
            .0
            .iter()
            .map(|row| {
                let mut columns = row
                    .iter()
                    .map(|opt_value| match opt_value {
                        None => Value::Null,
                        Some(value) => Value::from(*value),
                    })
                    .collect::<Vec<_>>();
                while matches!(columns.last(), Some(Null)) {
                    columns.pop();
                }
                Array(columns)
            })
            .collect::<Vec<_>>();
        while matches!(rows.last(), Some(Array(slice)) if slice.is_empty()) {
            rows.pop();
        }
        Array(rows)
    }
}

impl<const C: usize, const R: usize, T> TryFrom<&Value> for Table<C, R, T>
where
    T: for<'a> TryFrom<&'a Value, Error = String> + Copy,
{
    type Error = String;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        let rows = value.as_array().ok_or("Table must be an array")?;
        if rows.len() > R {
            return Err(format!("Table has too many rows (maximum is {})", R));
        }
        let mut table = Self::default();
        for (r, row) in rows.iter().enumerate() {
            let row = row
                .as_array()
                .ok_or_else(|| format!("Row {} must be an array", r))?;
            if row.len() > C {
                return Err(format!("Row {} has too many columns (maximum is {})", r, C));
            }
            for (c, cell) in row.iter().enumerate() {
                table.0[r][c] = if cell.is_null() {
                    None
                } else {
                    Some(
                        T::try_from(cell)
                            .map_err(|e| format!("Invalid cell ({}, {}): {}", r, c, e))?,
                    )
                }
            }
        }
        Ok(table)
    }
}

impl<const C: usize, const R: usize, T> Default for Table<C, R, T> {
    fn default() -> Self {
        Self(array::from_fn(|_| array::from_fn(|_| None)))
    }
}
