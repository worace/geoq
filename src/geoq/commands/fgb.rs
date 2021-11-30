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
    let feats = stdin_features()?;
    let buffer = fgb::write(feats);
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
use geozero::geojson::GeoJsonWriter;
use geozero::ToJson;

fn read(path: &str) -> Result<(), Error> {
    let mut file = BufReader::new(File::open(path)?);
    let mut fgb = FgbReader::open(&mut file)?;
    eprintln!("{:?}", fgb.header());
    fgb.select_all()?;

    // while let Some(feature) = fgb.next()? {
    //     dbg!(feature.properties()?);
    //     println!("{}", feature.to_json()?);
    // }

    let mut json_data: Vec<u8> = Vec::new();
    let mut json = GeoJsonWriter::new(&mut json_data);
    fgb.process_features(&mut json)?;
    println!("{}", std::str::from_utf8(&json_data)?);
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
