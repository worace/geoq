use crate::geoq::error::Error;
use once_cell::sync::Lazy;
use regex::Regex;
use std::fmt;

static LATLON: Lazy<Regex> = Lazy::new(|| Regex::new(r"^-?\d+\.?\d*[,\t]-?\d+\.?\d*$").unwrap());
static GH: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)^[0-9a-z--a--i--l--o]+$").unwrap());
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
}

impl fmt::Display for Input {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Input::LatLon(ref raw) => write!(f, "LatLon({})", raw),
            Input::Geohash(ref raw) => write!(f, "Geohash({})", raw),
            Input::WKT(ref raw) => write!(f, "WKT({})", raw),
            Input::GeoJSON(ref raw) => write!(f, "GeoJSON({})", raw),
        }
    }
}

pub fn read_line(line: String) -> Result<Input, Error> {
    if LATLON.is_match(&line) {
        Ok(Input::LatLon(line))
    } else if GH.is_match(&line) {
        Ok(Input::Geohash(line))
    } else if JSON.is_match(&line) {
        Ok(Input::GeoJSON(line))
    } else if WKT.is_match(&line) {
        Ok(Input::WKT(line))
    } else {
        Err(Error::UnknownEntityFormat)
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
