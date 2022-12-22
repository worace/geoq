use crate::geoq::error::Error;
use once_cell::sync::Lazy;
use regex::Regex;
use std::fmt;

static LATLON: Lazy<Regex> = Lazy::new(|| Regex::new(r"^-?\d+\.?\d*[,\t]-?\d+\.?\d*$").unwrap());
static GH: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)^[0-9a-z--a--i--l--o]+$").unwrap());
static H3: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)^0?[0-9a-f]{15,16}$").unwrap());
static JSON: Lazy<Regex> = Lazy::new(|| Regex::new(r"\{").unwrap());
static WKT: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?ix)^point|linestring|polygon|multipoint|multilinestring|multipolygon").unwrap()
});

#[derive(Debug, Clone)]
pub enum Input {
    LatLon(String),
    Geohash(String),
    WKT(String),
    GeoJSON(String),
    H3(String),
}

impl fmt::Display for Input {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Input::LatLon(ref raw) => write!(f, "LatLon({})", raw),
            Input::Geohash(ref raw) => write!(f, "Geohash({})", raw),
            Input::WKT(ref raw) => write!(f, "WKT({})", raw),
            Input::GeoJSON(ref raw) => write!(f, "GeoJSON({})", raw),
            Input::H3(ref raw) => write!(f, "H3Cell({})", raw),
        }
    }
}

pub fn read_line(line: String) -> Result<Input, Error> {
    if LATLON.is_match(&line) {
        Ok(Input::LatLon(line))
    } else if H3.is_match(&line) {
        Ok(Input::H3(line))
    } else if GH.is_match(&line) {
        Ok(Input::Geohash(line))
    } else if JSON.is_match(&line) {
        Ok(Input::GeoJSON(line))
    } else if WKT.is_match(&line) {
        Ok(Input::WKT(line))
    } else {
        Err(Error::InvalidInput(format!(
            "Unable to parse single-line input: {}",
            line
        )))
    }
}

#[test]
fn reading_input_formats() {
    match read_line("12,34".to_string()) {
        Ok(Input::LatLon(_)) => assert!(true),
        _ => assert!(false),
    }
    match read_line("12\t34".to_string()) {
        Ok(Input::LatLon(_)) => assert!(true),
        _ => assert!(false),
    }
}

#[test]
fn h3_regex_format() {
    assert!(H3.is_match("862749967ffffff"));
    assert!(H3.is_match("8027ffffffffffff"));
    // too short
    assert!(!H3.is_match("8027fffffffff"));
    // too long
    assert!(!H3.is_match("8027fffffffffffff"));
    // optional leading 0
    assert!(H3.is_match("08027ffffffffffff"));
}
