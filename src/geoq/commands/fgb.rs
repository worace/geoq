use crate::geoq::{error::Error, fgb, reader::Reader};
use clap::ArgMatches;
use flatgeobuf::FgbReader;
use std::fs::File;
use std::io::{self, BufReader};
use std::path::Path;

fn stdin_features() -> Result<Vec<geojson::Feature>, Error> {
    let mut input_features: Vec<geojson::Feature> = Vec::new();

    let stdin = io::stdin();
    let mut stdin_reader = stdin.lock();
    let reader = Reader::new(&mut stdin_reader);

    for e_res in reader {
        match e_res {
            Err(e) => return Err(e),
            Ok(e) => {
                input_features.push(e.geojson_feature());
            }
        }
    }
    Ok(input_features)
}

fn write(path: &str) -> Result<(), Error> {
    let buffer = fgb::write(&stdin_features()?);
    let res = std::fs::write(Path::new(path), buffer);
    match res {
        Ok(_) => Ok(()),
        Err(_) => Err(Error::ProgramError(format!(
            "Error writing flatgeobuf data to file {}",
            path
        ))),
    }
}

use flatgeobuf::*;
use geozero::ToJson;

fn read(path: &str) -> Result<(), Error> {
    let mut file = BufReader::new(File::open(path)?);
    let mut fgb = FgbReader::open(&mut file)?;
    eprintln!("{:?}", fgb.header());
    fgb.select_all()?;
    while let Some(feature) = fgb.next()? {
        println!("{}", feature.to_json()?);
    }
    Ok(())
}

pub fn run(m: &ArgMatches) -> Result<(), Error> {
    match m.subcommand() {
        ("write", Some(args)) => {
            let path: &str = args.value_of("path").unwrap();
            write(path)
        }
        ("read", Some(args)) => {
            let path: &str = args.value_of("path").unwrap();
            read(path)
        }
        _ => Err(Error::UnknownCommand),
    }
}
