#[derive(Debug)]
pub enum Error {
    InvalidGeoJSON,
    NotImplemented,
    UnknownCommand,
    UnknownEntityFormat,
    InvalidWkt,
}
