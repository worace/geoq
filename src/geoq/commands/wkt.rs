use std::io;
use geoq::reader::Reader;
use geoq::entity;
use geoq::error::Error;

pub fn run_wkt() -> Result<(), Error> {
    let stdin = io::stdin();
    let mut stdin_reader = stdin.lock();
    let reader = Reader::new(&mut stdin_reader);
    let entities = reader.flat_map(|i| entity::from_input(i));
    for e in entities {
        println!("{}", e.wkt());
    }
    Ok(())
}
