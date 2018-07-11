use geoq::error::Error;
use geoq::reader::Reader;
use std::io;

pub fn run() -> Result<(), Error> {
    let stdin = io::stdin();
    for input in Reader::new(&mut stdin.lock()) {
        println!("{}", input);
    }
    Ok(())
}
