use geoq::reader;
use geoq::error::Error;

pub fn run() -> Result<(), Error> {
    reader::for_entity(|e| Ok(println!("{}", e.wkt())))
}
