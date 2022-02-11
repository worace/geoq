use std::{convert::From, io, str::Utf8Error};

use geozero::error::GeozeroError;

#[derive(Debug)]
pub enum Error {
    // InvalidLatLon,
    InvalidGeoJSON,
    NotImplemented,
    UnknownCommand,
    UnknownEntityFormat,
    InvalidWkt,
    MissingArgument,
    InvalidNumberFormat(String),
    InputTooLarge,
    IOError,
    JSONParseError,
    InvalidJSONType,
    PolygonRequired,
    IPGeolocationError,
    HTTPError,
    TooManyFeatures,
    PointRequired,
    DistanceFailed,
    InvalidGeohashPoint,
    NoInputGiven,
    ShapefileReaderError(String),
    ProgramError(String),
    InvalidInput(String),
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

impl From<GeozeroError> for Error {
    fn from(e: GeozeroError) -> Self {
        Error::ProgramError(format!("{}", e))
    }
}

impl From<std::str::Utf8Error> for Error {
    fn from(e: Utf8Error) -> Self {
        Error::ProgramError(format!("{}", e))
    }
}
