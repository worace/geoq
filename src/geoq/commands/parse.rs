use geoq::error::Error;
use std::io;
use std::io::BufRead;
use std::io::Read;
use std::str;
use geoq::parser;
use nom::error::VerboseError;
use nom::Err::Incomplete;
use nom::error::ErrorKind;

pub fn run() -> Result<(), Error> {
    let mut buffer = [0; 1024];
    while let Ok(bytes_read) = io::stdin().read(&mut buffer) {
        if bytes_read == 0 { break; }
        match str::from_utf8(&buffer) {
            Ok(v) => {
                let res = parser::root::<(&str, ErrorKind)>(&v);
                println!("{:?}", res);
            },
            Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
        };
    }
    println!("out of input");


    Ok(())
}
