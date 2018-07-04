use std::io::BufRead;
use geoq::input;
use geoq::input::Input;

struct Reader<'a> {
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
        Ok(_) => Some(buf),
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

// pub type Reader = Iterator<Item = >;

// impl IntoIterator for BufRead {
//     type Item = Input;
//     type IntoIter = std::iter::Iterator<Result<Input, &'static str>>;

//     fn into_iter(self) -> Self::IntoIter {
//         self.lock().lines().map(|l| input::read_line(l))
//     }
// }

// pub fn new(reader: BufRead) -> IntoIterator<Item = Result<Input, &'static str>> {
    // let res = vec![];
    // res.into_iter()
    // // iter::empty::<Result<Input, &'static str>>()
// }

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
}
