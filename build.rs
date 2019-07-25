use std::env;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

use csv;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct OwnedRecord {
    tag3: String,
    tag1: Option<String>,
    name: Option<String>,
    autonym: Option<String>,
    source: Option<String>,
}

impl OwnedRecord {
    fn create_as_str(self) -> String {
        format!(
            "Record {{ tag3: \"{}\", tag1: {}, name: {}, autonym: {}, source: {} }}",
            &self.tag3,
            option_str(self.tag1),
            option_str(self.name),
            option_str(self.autonym),
            option_str(self.source),
        )
    }
}

fn option_str(opt: Option<String>) -> String {
    match opt {
        Some(val) => format!("Some(\"{}\")", val),
        None => "None".to_owned(),
    }
}

fn main() {
    let path = Path::new(&env::var("OUT_DIR").unwrap()).join("autonyms.rs");
    let mut file = BufWriter::new(File::create(&path).unwrap());

    write!(
        &mut file,
        "static LANGUAGE_AUTONYMS: phf::Map<&'static str, Record> = "
    )
    .unwrap();

    let autonyms_tsv = include_str!("assets/iso639-autonyms.tsv");
    let mut reader = csv::ReaderBuilder::new()
        .delimiter(b'\t')
        .from_reader(autonyms_tsv.as_bytes());

    let mut builder = phf_codegen::Map::new();

    reader
        .deserialize()
        .filter_map(|r: Result<OwnedRecord, csv::Error>| r.ok())
        .for_each(|record| {
            let key = record.tag1.clone().or(Some(record.tag3.clone())).unwrap();

            builder.entry(key, &record.create_as_str());
        });

    builder.build(&mut file).unwrap();
    write!(&mut file, ";\n").unwrap();
}
