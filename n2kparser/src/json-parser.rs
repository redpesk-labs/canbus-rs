
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

extern crate serde;
extern crate serde_json;
extern crate sockcan;

use sockcan::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::prelude::*;
use std::io;


#[derive(Serialize, Deserialize)]
enum PgnType {
    Single,
    #[serde(rename = "Lookup table")]
    LookupTable,
    Integer,
    #[serde(rename = "Binary data")]
    BinaryData,
    #[serde(rename = "Manufacturer code")]
    ManufacturerCode,
    ISO,
    Temperature,
    Time,
    Fast,
    Date,
    #[serde(rename = "ASCII or UNICODE string starting with length and control byte")]
    AsciiUnicode,
    #[serde(rename = "ASCII string starting with length byte")]
    AsciiString,
    #[serde(rename = "ASCII text")]
    AsciiText,
    Latitude,
    Longitude,
    Bitfield,
}

#[derive(Serialize, Deserialize)]
struct LookupEntry {
    name: String,
    value: u64,
}

#[derive(Serialize, Deserialize)]
struct FieldDef {
    Order: u8,
    Id: String,
    Description: Option<String>,
    BitLength: Option<u16>,
    BitLengthVariable: Option<bool>,
    BitOffset: Option<u16>,
    BitStart: Option<u16>,
    Type: Option<PgnType>,
    Resolution: Option<f64>,
    Signed: Option<bool>,
    RangeMin: Option<f64>,
    RangeMax: Option<f64>,
    EnumValues: Option<Vec<LookupEntry>>,
}

#[derive(Serialize, Deserialize)]
struct PgnDef {
    PGN: u32,
    Id: String,
    Description: String,
    Type: PgnType,
    Complete: bool,
    Length: Option<u8>,
    Fields: Vec<FieldDef>,
}

#[derive(Serialize, Deserialize)]
pub struct CanBoat {
    Comment: Option<String>,
    Licence: Option<String>,
    PGNs: Vec<PgnDef>,
}

pub fn from_file(dbcpath: &str) -> Result<CanBoat, CanError> {
    let filename = Box::leak(dbcpath.to_owned().into_boxed_str()) as &'static str;
    let dbc_buffer = || -> Result<Vec<u8>, io::Error> {
        let mut fd = File::open(filename)?;
        let size = fd.metadata().unwrap().len();
        let mut buffer = Vec::with_capacity(size as usize);
        fd.read_to_end(&mut buffer)?;
        Ok(buffer)
    };

    match dbc_buffer() {
        Err(error) => Err(CanError::new(filename,error.to_string())),
        Ok(buffer) => {
            let slice = buffer.leak();
            let data = std::str::from_utf8(slice).unwrap();
            let pgns: CanBoat= serde_json::from_str::<CanBoat>(data).unwrap();
            Ok(pgns)
        }
    }
}

fn main() -> Result<(), CanError> {

    let pngs= from_file("n2kparser/pgn/canboat-pgns.json") ?;

    Ok(())
}