use crate::geoq::{error::Error, par};

pub fn run() -> Result<(), Error> {
    par::for_stdin_entity(|e| Ok(vec![format!("{}", e.wkt())]))
}