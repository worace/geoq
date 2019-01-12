extern crate serde_json;

use std::io;
use std::convert::From;

#[derive(Debug)]
pub enum Error {
    // InvalidLatLon,
    InvalidGeoJSON,
    NotImplemented,
    UnknownCommand,
    UnknownEntityFormat,
    InvalidWkt,
    MissingArgument,
    InvalidNumberFormat,
    // InputTooLarge,
    IOError,
    JSONParseError,
    InvalidJSONType,
    PolygonRequired,
    IPGeolocationError,
    HTTPError,
    TooManyFeatures
}

impl From<io::Error> for Error {
    fn from(_: io::Error) -> Self {
        Error::IOError
    }
}

impl From<serde_json::Error> for Error {
    fn from(_: serde_json::Error) -> Self {
        Error::JSONParseError
    }
}

impl From<reqwest::Error> for Error {
    fn from(_: reqwest::Error) -> Self {
        Error::HTTPError
    }
}
