use crate::geoq::{entity::Entity, error::Error, reader::Reader};
use clap::ArgMatches;
use flatbuffers::{FlatBufferBuilder, UOffsetT, WIPOffset};
use flatgeobuf::{Feature, Header, HeaderBuilder};
use std::io;

// https://www.notion.so/worace/Flatgeobuf-4c2eb8ea1475419991863f36bd2fa355

fn write_fgb_feature(bldr: &mut FlatBufferBuilder, entity: &Entity) -> UOffsetT {
    // flatgeobuf::GeometryOffset
    let args = flatgeobuf::FeatureArgs {
        columns: None,
        geometry: None,
        properties: None,
    };
    let offset = flatgeobuf::Feature::create(bldr, &args);
    offset.value()
}

// table Header {
//   name: string;                 // Dataset name
//   envelope: [double];           // Bounds
//   geometry_type: GeometryType;  // Geometry type (should be set to Unknown if per feature geometry type)
//   has_z: bool = false;           // Does geometry have Z dimension?
//   has_m: bool = false;           // Does geometry have M dimension?
//   has_t: bool = false;           // Does geometry have T dimension?
//   has_tm: bool = false;          // Does geometry have TM dimension?
//   columns: [Column];            // Attribute columns schema (can be omitted if per feature schema)
//   features_count: ulong;        // Number of features in the dataset (0 = unknown)
//   index_node_size: ushort = 16; // Index node size (0 = no index)
//   crs: Crs;                     // Spatial Reference System
//   title: string;                // Dataset title
//   description: string;          // Dataset description (intended for free form long text)
//   metadata: string;             // Dataset metadata (intended to be application specific and suggested to be structured fx. JSON)
// }
fn write_header<'a>(
    bldr: &'a mut FlatBufferBuilder,
    features: Vec<geojson::Feature>,
) -> WIPOffset<Header<'a>> {
    // https://github.com/flatgeobuf/flatgeobuf/blob/master/src/fbs/header.fbs
    let name = bldr.create_string("Geoq-generated FGB");
    let mut hb = HeaderBuilder::new(bldr);
    hb.add_name(name);
    hb.finish()
}

fn write() -> Result<(), Error> {
    // collect features into vector
    // read features to get header schema (Columns "table")
    // generate + write header
    // iterate + convert + write each feature
    let mut buffer: Vec<u8> = Vec::new();
    let mut features: Vec<Feature> = Vec::new();
    let mut builder = FlatBufferBuilder::new();

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

    // write_fgb_feature(&mut builder, &e);
    Ok(())
}

pub fn run(_fgb: &ArgMatches) -> Result<(), Error> {
    write()
}
