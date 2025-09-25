use core::{
    error::Error,
    fmt::{self, Display},
    iter,
};

use std::{
    fs::File,
    io::{self, BufReader, Read},
    path::Path,
    sync::LazyLock,
};

use csv::StringRecord;

use termcolor::{Color, ColorSpec, WriteColor};

use crate::{dsv::get_tsv_reader, ui::styles::WriteStyled, util::strings::unescape};

pub static STYLE_UNIGRAM_KEY: LazyLock<ColorSpec> = LazyLock::new(|| {
    let mut color_spec = ColorSpec::new();
    color_spec.set_bold(true);
    color_spec.set_fg(Some(Color::Yellow));
    color_spec
});

pub static STYLE_BIGRAM_KEY: LazyLock<ColorSpec> = LazyLock::new(|| {
    let mut color_spec = ColorSpec::new();
    color_spec.set_bold(true);
    color_spec.set_fg(Some(Color::Blue));
    color_spec
});

pub static STYLE_TRIGRAM_KEY: LazyLock<ColorSpec> = LazyLock::new(|| {
    let mut color_spec = ColorSpec::new();
    color_spec.set_bold(true);
    color_spec.set_fg(Some(Color::Magenta));
    color_spec
});

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct UnigramKey(u8);

impl UnigramKey {
    pub fn as_u8(&self) -> u8 {
        self.0
    }

    pub fn as_usize(&self) -> usize {
        self.0 as usize
    }
}

impl Display for UnigramKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0 as char)
    }
}

impl From<u8> for UnigramKey {
    fn from(value: u8) -> Self {
        Self(value)
    }
}

impl From<UnigramKey> for usize {
    fn from(value: UnigramKey) -> Self {
        value.as_usize()
    }
}

impl TryFrom<&str> for UnigramKey {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value.len() == 1 && value.is_ascii() {
            let bytes = value.as_bytes();
            Ok(UnigramKey::from(bytes[0]))
        } else {
            Err(format!("Invalid unigram key '{}'", value))
        }
    }
}

impl WriteStyled for UnigramKey {
    fn write_styled(&self, writer: &mut dyn WriteColor) -> io::Result<()> {
        writer.set_color(&STYLE_UNIGRAM_KEY)?;
        write!(writer, "{}", self.as_u8() as char)?;
        writer.reset()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct BigramKey(u16);

impl BigramKey {
    pub fn as_u8_pair(&self) -> (u8, u8) {
        ((self.0 >> 8) as u8, self.0 as u8)
    }

    pub fn as_u16(&self) -> u16 {
        self.0
    }

    pub fn as_usize(&self) -> usize {
        self.0 as usize
    }
}

impl Display for BigramKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let (b1, b2) = self.as_u8_pair();
        write!(f, "{}{}", b1 as char, b2 as char)
    }
}

impl From<(u8, u8)> for BigramKey {
    fn from(value: (u8, u8)) -> Self {
        let (b1, b2) = value;
        Self((b1 as u16) << 8 | b2 as u16)
    }
}

impl From<BigramKey> for usize {
    fn from(value: BigramKey) -> Self {
        value.as_usize()
    }
}

impl TryFrom<&str> for BigramKey {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value.len() == 2 && value.is_ascii() {
            let bytes = value.as_bytes();
            Ok(BigramKey::from((bytes[0], bytes[1])))
        } else {
            Err(format!("Invalid bigram key '{}'", value))
        }
    }
}

impl WriteStyled for BigramKey {
    fn write_styled(&self, writer: &mut dyn WriteColor) -> io::Result<()> {
        writer.set_color(&STYLE_BIGRAM_KEY)?;
        write!(writer, "{}", self)?;
        writer.reset()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct TrigramKey(u32);

impl TrigramKey {
    pub fn as_u8_triple(&self) -> (u8, u8, u8) {
        ((self.0 >> 16) as u8, (self.0 >> 8) as u8, self.0 as u8)
    }

    pub fn as_u32(&self) -> u32 {
        self.0
    }

    pub fn as_usize(&self) -> usize {
        self.0 as usize
    }
}

impl Display for TrigramKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let (b1, b2, b3) = self.as_u8_triple();
        write!(f, "{}{}{}", b1 as char, b2 as char, b3 as char)
    }
}

impl From<(u8, u8, u8)> for TrigramKey {
    fn from(value: (u8, u8, u8)) -> Self {
        let (b1, b2, b3) = value;
        Self((b1 as u32) << 16 | (b2 as u32) << 8 | b3 as u32)
    }
}

impl From<TrigramKey> for usize {
    fn from(value: TrigramKey) -> Self {
        value.as_usize()
    }
}

impl TryFrom<&str> for TrigramKey {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value.len() == 3 && value.is_ascii() {
            let bytes = value.as_bytes();
            Ok(TrigramKey::from((bytes[0], bytes[1], bytes[2])))
        } else {
            Err(format!("Invalid trigram key '{}'", value))
        }
    }
}

impl WriteStyled for TrigramKey {
    fn write_styled(&self, writer: &mut dyn WriteColor) -> io::Result<()> {
        writer.set_color(&STYLE_TRIGRAM_KEY)?;
        write!(writer, "{}", self)?;
        writer.reset()
    }
}

pub type UnigramTable = [u64; 1 << 8];
pub type BigramTable = [u64; 1 << 16];
pub type TrigramTable = [u64; 1 << 24];

pub fn read_unigram_table<R: Read>(reader: R) -> Result<Box<UnigramTable>, Box<dyn Error>> {
    read_ngram_table(reader, |s| UnigramKey::try_from(s))
}

pub fn read_unigram_table_from_bytes(
    bytes: &'static [u8],
) -> Result<Box<UnigramTable>, Box<dyn Error>> {
    read_unigram_table(BufReader::new(bytes))
}

pub fn read_unigram_table_from_path(path: &Path) -> Result<Box<UnigramTable>, Box<dyn Error>> {
    read_unigram_table(BufReader::new(File::open(path)?))
}

pub fn read_bigram_table<R: Read>(reader: R) -> Result<Box<BigramTable>, Box<dyn Error>> {
    read_ngram_table(reader, |s| BigramKey::try_from(s))
}

pub fn read_bigram_table_from_bytes(
    bytes: &'static [u8],
) -> Result<Box<BigramTable>, Box<dyn Error>> {
    read_bigram_table(BufReader::new(bytes))
}

pub fn read_bigram_table_from_path(path: &Path) -> Result<Box<BigramTable>, Box<dyn Error>> {
    read_bigram_table(BufReader::new(File::open(path)?))
}

pub fn read_trigram_table<R: Read>(reader: R) -> Result<Box<TrigramTable>, Box<dyn Error>> {
    read_ngram_table(reader, |s| TrigramKey::try_from(s))
}

pub fn read_trigram_table_from_bytes(
    bytes: &'static [u8],
) -> Result<Box<TrigramTable>, Box<dyn Error>> {
    read_trigram_table(BufReader::new(bytes))
}

pub fn read_trigram_table_from_path(path: &Path) -> Result<Box<TrigramTable>, Box<dyn Error>> {
    read_trigram_table(BufReader::new(File::open(path)?))
}

pub fn sum_ngram_table<T: Copy + iter::Sum<T>>(slice: &[T]) -> T {
    slice.iter().copied().sum()
}

fn read_ngram_table<const N: usize, K: Into<usize>, R: Read>(
    reader: R,
    key_fn: impl Fn(&str) -> Result<K, String>,
) -> Result<Box<[u64; N]>, Box<dyn Error>> {
    // NOTE This can cause a stack overflow for large values of N.
    // let mut array = Box::new([0u64; N]);
    let mut array: Box<[u64; N]> = vec![0u64; N]
        .into_boxed_slice()
        .try_into()
        .map_err(|_| format!("Unable to allocate an array of {} elements", N))?;
    for result in get_tsv_reader(reader).records() {
        let record: StringRecord = result?;
        let key_str = unescape::<true>(record.get(0).ok_or("Missing key column")?)?;
        // NOTE Characters must be ASCII, and the control characters SOH, STX, and ETX are reserved.
        if key_str
            .chars()
            .all(|ch| ch == '\0' || ('\x04'..='\x7f').contains(&ch))
        {
            let key = key_fn(&key_str)?;
            let value_str = record.get(1).ok_or("Missing value column")?;
            let value: u64 = value_str.parse().map_err(|e| {
                format!("Invalid value '{}' for key '{}': {}", value_str, key_str, e)
            })?;
            array[key.into()] = value;
        }
    }
    Ok(array)
}
