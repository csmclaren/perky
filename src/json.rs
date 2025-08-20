use core::error::Error;

use std::io::{self, Read, Write};

use serde::{Serialize, de::DeserializeOwned};

use serde_json::{Serializer, Value, from_reader, ser::PrettyFormatter};

pub fn read_enveloped_data<R: Read, T: DeserializeOwned>(
    reader: R,
    expected_version: u64,
) -> Result<T, Box<dyn Error>> {
    let value = read_json::<_, Value>(reader)?;
    let object = value.as_object().ok_or("Expected top-level JSON object")?;
    let version = object
        .get("version")
        .ok_or("Expected 'version' field")?
        .as_u64()
        .ok_or("Value of 'version' field must be of type 'u64'")?;
    if version != expected_version {
        Err(format!("Unsupported version: {}", version))?;
    }
    let data = T::deserialize(object.get("data").ok_or("Expected 'data' field")?)?;
    Ok(data)
}

pub fn read_json<R: Read, T: DeserializeOwned>(reader: R) -> Result<T, serde_json::Error> {
    from_reader(reader)
}

pub fn write_json<W: Write, T: Serialize>(writer: W, value: &T) -> Result<(), serde_json::Error> {
    let mut serializer = Serializer::with_formatter(writer, PrettyFormatter::new());
    value.serialize(&mut serializer)
}

pub fn write_json_flatten_primitive_arrays<const N: usize, W: Write + ?Sized>(
    writer: &mut W,
    value: &Value,
    indent: usize,
) -> io::Result<()> {
    use Value::*;
    fn is_primitive_array(slice: &[Value]) -> bool {
        slice
            .iter()
            .all(|value| matches!(value, Null | Bool(_) | Number(_) | String(_)))
    }
    fn write_spaces<W: Write + ?Sized>(writer: &mut W, count: usize) -> io::Result<()> {
        write!(writer, "{}", " ".repeat(count))
    }
    match value {
        Array(vec) => {
            if is_primitive_array(vec) {
                write!(writer, "[")?;
                for (i, val) in vec.iter().enumerate() {
                    if i != 0 {
                        write!(writer, ", ")?;
                    }
                    write_json_flatten_primitive_arrays::<N, _>(writer, val, indent)?;
                }
                write!(writer, "]")
            } else {
                writeln!(writer, "[")?;
                for (i, val) in vec.iter().enumerate() {
                    if i != 0 {
                        writeln!(writer, ",")?;
                    }
                    write_spaces(writer, indent + N)?;
                    write_json_flatten_primitive_arrays::<N, _>(writer, val, indent + N)?;
                }
                writeln!(writer)?;
                write_spaces(writer, indent)?;
                write!(writer, "]")
            }
        }
        Object(map) => {
            writeln!(writer, "{{")?;
            let mut first = true;
            for (k, v) in map {
                if !first {
                    writeln!(writer, ",")?;
                } else {
                    first = false;
                }
                write_spaces(writer, indent + N)?;
                write!(writer, "\"{}\": ", k)?;
                write_json_flatten_primitive_arrays::<N, _>(writer, v, indent + N)?;
            }
            writeln!(writer)?;
            write_spaces(writer, indent)?;
            write!(writer, "}}")
        }
        _ => {
            write!(writer, "{}", value)
        }
    }
}
