use std::io::{BufReader, Read};

use csv::Reader;

pub fn get_tsv_reader<R: Read>(reader: R) -> Reader<BufReader<R>> {
    csv::ReaderBuilder::new()
        .delimiter(b'\t')
        .has_headers(false)
        .quoting(false)
        .from_reader(BufReader::new(reader))
}
