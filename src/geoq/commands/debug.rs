use geoq::error::Error;
use geoq::reader;

pub fn run() -> Result<(), Error> {
    reader::for_input(|i| Ok(println!("{}", i)))
}
