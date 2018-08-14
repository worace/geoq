use geoq::error::Error;
use geoq::reader;

pub fn run() -> Result<(), Error> {
    reader::for_entity(|e| Ok(println!("{}", e)))
}
