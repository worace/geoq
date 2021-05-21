use crate::geoq::{error::Error, par, bbox};

pub fn run() -> Result<(), Error> {
    par::for_stdin_entity(|e| {
        // bbox::bbox(e.)
        Ok(vec![format!("{}", e.wkt())])
    })
}
