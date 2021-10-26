use crate::geoq::{entity::Entity, error::Error, fgb, fgb::header::ColSpec, reader::Reader};
use clap::ArgMatches;
use flatbuffers::{FlatBufferBuilder, ForwardsUOffset, UOffsetT, Vector, WIPOffset};
use flatgeobuf::{
    Column, ColumnArgs, ColumnBuilder, ColumnType, Feature, FeatureArgs, FgbReader, GeometryType,
    Header, HeaderBuilder,
};
use serde_json::Map;
use std::convert::{TryFrom, TryInto};
use std::fs::File;
use std::io::{self, BufReader};
use std::path::Path;

// table Feature {
//   geometry: Geometry;  // Geometry
//   properties: [ubyte]; // Custom buffer, variable length collection of key/value pairs (key=ushort)
//   columns: [Column];   // Attribute columns schema (optional)
// }
fn write_feature(
    bldr: &mut FlatBufferBuilder,
    col_specs: &Vec<ColSpec>,
    f: &geojson::Feature,
) -> () {
    eprintln!("Write geojson feature: {:?}", f);
    // https://github.com/flatgeobuf/flatgeobuf/blob/master/src/ts/generic/feature.ts#L47-L143
    // flatgeobuf::GeometryOffset

    // Q: should this repeat all columns for the schema, or only the ones that apply to this feature?
    let cols_vec = fgb::columns::build(bldr, col_specs);
    let props =
        fgb::properties::feature_props(f, col_specs).map(|bytes| bldr.create_vector(&bytes[..]));

    // Geometry serialization
    // https://github.com/flatgeobuf/flatgeobuf/blob/master/src/ts/generic/geometry.ts#L37-L64
    let geom = fgb::geometry::build(bldr, f);

    let args = flatgeobuf::FeatureArgs {
        columns: Some(cols_vec),
        geometry: Some(geom),
        properties: props,
    };
    let offset = flatgeobuf::Feature::create(bldr, &args);

    bldr.finish_size_prefixed(offset, None);
}

fn write(path: &str) -> Result<(), Error> {
    // collect features into vector
    // read features to get header schema (Columns "table")
    // generate + write header
    // iterate + convert + write each feature
    let mut _buffer: Vec<u8> = Vec::new();
    let mut _features: Vec<Feature> = Vec::new();

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

    // Binary Layout
    // MB: Magic bytes (0x6667620366676201)
    // H: Header (variable size flatbuffer) (written as its own standalone flatbuffer)
    // I (optional): Static packed Hilbert R-tree index (static size custom buffer)
    // DATA: Features (each written as its own standalone flatbuffer?)

    let mut buffer: Vec<u8> = vec![0x66, 0x67, 0x62, 0x03, 0x66, 0x67, 0x62, 0x00];

    let mut header_builder = FlatBufferBuilder::new();
    let col_specs = fgb::header::write_header(&mut header_builder, &input_features);
    // Header is now done, use header_builder.finished_data() to access &[u8]
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
        let mut builder = FlatBufferBuilder::new();
        write_feature(&mut builder, &col_specs, &f);
        buffer.extend(builder.finished_data());
    }

    // write_fgb_feature(&mut builder, &e);
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
