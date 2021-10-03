use crate::geoq::{entity::Entity, error::Error, reader::Reader};
use clap::ArgMatches;
use flatbuffers::{FlatBufferBuilder, UOffsetT, WIPOffset};
use flatgeobuf::{
    Column, ColumnArgs, ColumnBuilder, ColumnType, Feature, GeometryType, Header, HeaderBuilder,
};
use std::collections::HashSet;
use std::convert::TryInto;
use std::io;

// https://www.notion.so/worace/Flatgeobuf-4c2eb8ea1475419991863f36bd2fa355

fn write_fgb_feature(
    bldr: &mut FlatBufferBuilder,
    cols: Vec<ColSpec>,
    f: &geojson::Feature,
) -> UOffsetT {
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

fn geometry_type(features: &Vec<geojson::Feature>) -> GeometryType {
    let mut types = HashSet::new();
    let mut last_gtype = GeometryType::Unknown;
    for f in features {
        // let val = f.geometry.map(|g| g.value);
        if let Some(geom) = &f.geometry {
            let gtype = match geom.value {
                geojson::Value::Point(_) => GeometryType::Point,
                geojson::Value::LineString(_) => GeometryType::LineString,
                geojson::Value::Polygon(_) => GeometryType::Polygon,
                geojson::Value::MultiPoint(_) => GeometryType::MultiPoint,
                geojson::Value::MultiLineString(_) => GeometryType::MultiLineString,
                geojson::Value::MultiPolygon(_) => GeometryType::MultiPolygon,
                geojson::Value::GeometryCollection(_) => GeometryType::GeometryCollection,
            };
            types.insert(gtype);
            last_gtype = gtype;
        }
    }

    if types.len() == 1 {
        last_gtype
    } else {
        GeometryType::Unknown
    }
}

#[derive(Clone)]
struct ColSpec {
    name: String,
    type_: ColumnType,
}

fn col_specs(features: &Vec<geojson::Feature>) -> Vec<ColSpec> {
    vec![ColSpec {
        name: "properties".to_string(),
        type_: ColumnType::Json,
    }]
}

fn write_header<'a>(
    bldr: &'a mut FlatBufferBuilder,
    features: &Vec<geojson::Feature>,
) -> (&'a [u8], Vec<ColSpec>) {
    // https://github.com/flatgeobuf/flatgeobuf/blob/master/src/fbs/header.fbs
    // https://github.com/flatgeobuf/flatgeobuf/blob/master/src/ts/generic/featurecollection.ts#L158-L182
    let name = bldr.create_string("Geoq-generated FGB");
    let desc = bldr.create_string("Geoq-generated FGB");

    let col_specs = col_specs(features);
    let cols: Vec<WIPOffset<Column>> = col_specs
        .clone()
        .into_iter()
        .map(|c| {
            let col_name = bldr.create_string(&c.name);
            let mut cb = ColumnBuilder::new(bldr);
            cb.add_type_(c.type_);
            cb.add_name(col_name);
            cb.add_nullable(true);
            cb.finish()
            // let col: WIPOffset<Column> = ;
        })
        .collect();
    let cols_vec = bldr.create_vector(&cols[..]);

    let mut hb = HeaderBuilder::new(bldr);
    hb.add_name(name);
    hb.add_description(desc);
    hb.add_features_count(features.len().try_into().unwrap()); // not sure when this would fail...i guess 128bit system?
    hb.add_columns(cols_vec);
    hb.add_geometry_type(geometry_type(features));
    hb.add_index_node_size(0); // No Index? (following ts example)
    let header = hb.finish();
    bldr.finish(header, None);
    (bldr.finished_data(), col_specs)
}

fn write() -> Result<(), Error> {
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

    let mut header_builder = FlatBufferBuilder::new();
    let (header_bytes, col_specs) = write_header(&mut header_builder, &input_features);

    // write_fgb_feature(&mut builder, &e);
    Ok(())
}

pub fn run(_fgb: &ArgMatches) -> Result<(), Error> {
    write()
}
