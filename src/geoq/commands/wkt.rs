use geoq::error::Error;
use geoq::par;

pub fn run() -> Result<(), Error> {
    par::for_stdin_entity(|e| Ok(vec![format!("{}", e.wkt())]))
}
