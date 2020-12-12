use crate::geoq::{error::Error, par, reader::Reader, entity::Entity};
use clap::ArgMatches;
use geojson::GeoJson;
use flatbuffers::{FlatBufferBuilder, UOffsetT};
use flatgeobuf::{Header, Feature};
use std::io;


fn write_fgb_feature(bldr: &mut FlatBufferBuilder, entity: &Entity) -> UOffsetT {
    flatgeobuf::GeometryOffset
    let args = flatgeobuf::FeatureArgs{
        columns: None,
        geometry: None,
        properties: None
    };
    let offset = flatgeobuf::Feature::create(bldr, &args);
    offset.value()
}

fn write() -> Result<(), Error> {
    // collect features into vector
    // read features to get header schema (Columns "table")
    // generate + write header
    // iterate + convert + write each feature
    let mut buffer: Vec<u8> = Vec::new();
    let mut features: Vec<Feature> = Vec::new();
    let mut builder = FlatBufferBuilder::new();

    let stdin = io::stdin();
    let mut stdin_reader = stdin.lock();
    let reader = Reader::new(&mut stdin_reader);
    for e_res in reader {
        match e_res {
            Err(e) => return Err(e),
            Ok(e) => {
                write_fgb_feature(&mut builder, &e);
            },
        }
    }

    Ok(())
}

pub fn run(_fgb: &ArgMatches) -> Result<(), Error> {
    write()
}
