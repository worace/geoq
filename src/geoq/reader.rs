extern crate geo_types;

use std::io;
use std::io::BufRead;
use geoq::input;
use geoq::input::Input;
use geoq::entity::{self, Entity};
use geoq::error::Error;

pub struct Reader<'a> {
    reader: &'a mut BufRead
}

impl<'a> Reader<'a> {
    pub fn new(reader: &'a mut BufRead) -> Reader<'a> {
        Reader{reader}
    }
}

fn read_line(buf_read: &mut BufRead) -> Option<String> {
    let mut buf = String::new();
    let bytes_read = buf_read.read_line(&mut buf);
    match bytes_read {
        Ok(0) => None,
        Ok(_) => {
            Some(buf.trim().to_string())
        },
        _ => None
    }
}

impl<'a> Iterator for Reader<'a> {
    type Item = Input;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(line) = read_line(&mut *self.reader) {
            Some(input::read_line(line))
        } else {
            None
        }
    }
}

pub fn inputs<F>(handler: F) -> Result<(), Error>
where F: Fn(Input) -> Result<(), Error> {
    let stdin = io::stdin();
    let mut reader = stdin.lock();
    while let Some(line) = read_line(&mut reader) {
        let input = input::read_line(line);
        if let Err(e) = handler(input) {
            return Err(e);
        }
    }
    Ok(())
}

pub fn entities<F>(handler: F) -> Result<(), Error>
where F: Fn(&mut Iterator<Item = Entity>) -> Result<(), Error>
{
    let stdin = io::stdin();
    let mut stdin_reader = stdin.lock();
    let reader = Reader::new(&mut stdin_reader);
    let mut entities = reader.flat_map(|i| entity::from_input(i));
    handler(&mut entities)
}

pub fn for_entity<F>(handler: F) -> Result<(), Error>
where F: Fn(Entity) -> Result<(), Error>
{
    entities(|e_iter| {
        let mut result = Ok(());
        for entity in e_iter {
            match handler(entity) {
                Ok(_) => continue,
                Err(e) => {
                    result = Err(e);
                    break
                }
            }
        }
        result
    })
}

pub fn for_input<F>(handler: F) -> Result<(), Error>
where F: Fn(Input) -> Result<(), Error>
{
    let stdin = io::stdin();
    let mut stdin_reader = stdin.lock();
    let reader = Reader::new(&mut stdin_reader);

    for input in reader {
        if let Err(e) = handler(input) {
            return Err(e);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use geoq::reader::Reader;

    #[test]
    fn test_reading_empty_string() {
        let mut pointer = "".as_bytes();
        let reader = Reader::new(&mut pointer);
        assert_eq!(0, reader.count());
    }

    #[test]
    fn test_reading_single_line() {
        let mut pointer = "9q5".as_bytes();
        let reader = Reader::new(&mut pointer);
        assert_eq!(1, reader.count());
    }

    #[test]
    fn test_checking_character_for_single_line() {
        let mut pointer = "9q5".as_bytes();
        let mut reader = Reader::new(&mut pointer);
        let gh = reader.next().unwrap();
        assert_eq!("9q5", gh.raw());
    }

    #[test]
    fn test_reading_2_lines() {
        let mut pointer = "9q5\n9q4".as_bytes();
        let mut reader = Reader::new(&mut pointer);
        let a = reader.next().unwrap();
        let b = reader.next().unwrap();
        assert_eq!("9q5", a.raw());
        assert_eq!("9q4", b.raw());
    }
}
