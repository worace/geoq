use crate::geoq::{error::Error, fgb, reader::Reader};
use clap::ArgMatches;
use flatgeobuf::FgbReader;
use geozero::GeozeroDatasource;
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

fn read(path: &str, bbox: Option<&str>) -> Result<(), Error> {
    let mut file = BufReader::new(File::open(path)?);
    let fgb = FgbReader::open(&mut file)?;

    let mut fgb = if let Some(bbox) = bbox {
        let parts: Vec<f64> = bbox
            .split(",")
            .map(|num| {
                num.parse::<f64>()
                    .expect("Invalid bbox format -- should be 4 comma-sepparated numbers")
            })
            .collect();
        if parts.len() != 4 {
            let e = Error::InvalidInput(format!("Invalid bounding box format: {}. Should be 4 comma-separated numbers: minX,minY,maxX,maxY.", bbox));
            return Err(e);
        }

        let (min_x, min_y, max_x, max_y) = (parts[0], parts[1], parts[2], parts[3]);
        fgb.select_bbox(min_x, min_y, max_x, max_y)?
    } else {
        fgb.select_all()?
    };

    let mut json_data: Vec<u8> = Vec::new();
    let mut json = GeoJsonWriter::new(&mut json_data);
    fgb.process(&mut json)?;
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
            let bbox: Option<&str> = args.value_of("bbox");
            read(path, bbox)
        }
        _ => Err(Error::UnknownCommand),
    }
}
