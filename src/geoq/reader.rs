extern crate geo_types;

use std::io;
use std::io::BufRead;
use geoq::input;
use geoq::entity::{self, Entity};
use geoq::error::Error;
use std::collections::VecDeque;
use std::iter::FromIterator;

pub struct Reader<'a> {
    reader: &'a mut BufRead,
    entities: VecDeque<Entity>
}

impl<'a> Reader<'a> {
    pub fn new(reader: &'a mut BufRead) -> Reader<'a> {
        Reader{reader, entities: VecDeque::new()}
    }
}

pub fn read_line(buf_read: &mut BufRead) -> Option<String> {
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
    type Item = Result<Entity, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(entity) = self.entities.pop_front() {
            return Some(Ok(entity));
        }

        while let Some(line) = read_line(&mut *self.reader) {
            match input::read_line(line) {
                Ok(i) => {
                    match entity::from_input(i) {
                        Ok(e_vec) => {
                            let mut entities = VecDeque::from_iter(e_vec);
                            if entities.is_empty() {
                                continue;
                            } else {
                                self.entities.append(&mut entities);
                                let e = self.entities.pop_front().unwrap();
                                return Some(Ok(e));
                            }
                        },
                        Err(e) => return Some(Err(e))
                    }
                },
                Err(e) => {
                    return Some(Err(e));
                }
            }
        }
        None
    }
}

pub fn entities<F>(handler: F) -> Result<(), Error>
where F: Fn(&mut Iterator<Item = Result<Entity, Error>>) -> Result<(), Error>
{
    let stdin = io::stdin();
    let mut stdin_reader = stdin.lock();
    let mut reader = Reader::new(&mut stdin_reader);
    handler(&mut reader)
}

pub fn for_entity<F>(handler: F) -> Result<(), Error>
where F: Fn(Entity) -> Result<(), Error>
{
    entities(|e_iter| {
        for e_res in e_iter {
            match e_res {
                Err(e) => return Err(e),
                Ok(entity) => {
                    if let Err(e) = handler(entity) {
                        return Err(e);
                    }
                }
            }
        }
        Ok(())
    })
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
        let gh = reader.next().unwrap().unwrap();
        assert_eq!("9q5", gh.raw());
    }

    #[test]
    fn test_reading_2_lines() {
        let mut pointer = "9q5\n9q4".as_bytes();
        let mut reader = Reader::new(&mut pointer);
        let a = reader.next().unwrap();
        let b = reader.next().unwrap();
        assert_eq!("9q5", a.unwrap().raw());
        assert_eq!("9q4", b.unwrap().raw());
    }
}
