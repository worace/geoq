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

// Binary Layout
// MB: Magic bytes (0x6667620366676201)
// H: Header (variable size flatbuffer) (written as its own standalone flatbuffer)
// I (optional): Static packed Hilbert R-tree index (static size custom buffer)
// DATA: Features (each written as its own standalone flatbuffer?)
fn write(path: &str) -> Result<(), Error> {
    // collect features into vector
    // read features to get header schema (Columns "table")
    // generate + write header
    // iterate + convert + write each feature
    let mut _buffer: Vec<u8> = Vec::new();
    let input_features = stdin_features()?;

    let mut buffer: Vec<u8> = vec![0x66, 0x67, 0x62, 0x03, 0x66, 0x67, 0x62, 0x00];

    let (header_builder, col_specs) = fgb::header::write(&input_features);
    buffer.extend(header_builder.finished_data());
    eprintln!("header data:");
    eprintln!("{:02X?}", header_builder.finished_data());
    eprintln!(
        "Writing {:?} bytes of header data",
        header_builder.finished_data().len()
    );

    for f in input_features {
        eprintln!("writing feature");
        dbg!(&f);
        let builder = fgb::feature::write(&col_specs, &f);
        buffer.extend(builder.finished_data());
    }

    let res = std::fs::write(Path::new(path), buffer);
    match res {
        Ok(_) => Ok(()),
        Err(_) => Err(Error::ProgramError(format!(
            "Error writing flatgeobuf data to file {}",
            path
        ))),
    }
}

use geozero::ProcessToJson;

fn read(path: &str) -> Result<(), Error> {
    let mut file = BufReader::new(File::open(path)?);
    let mut fgb = FgbReader::open(&mut file)?;
    eprintln!("{:?}", fgb.header());
    fgb.select_bbox(8.8, 47.2, 9.5, 55.3)?;
    println!("{}", fgb.to_json()?);
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
