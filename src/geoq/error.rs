use std::io;
use std::convert::From;

#[derive(Debug)]
pub enum Error {
    InvalidLatLon,
    InvalidGeoJSON,
    NotImplemented,
    UnknownCommand,
    UnknownEntityFormat,
    InvalidWkt,
    MissingArgument,
    InvalidNumberFormat,
    InputTooLarge,
    IOError
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Error::IOError
    }
}
