use super::columns;
use flatbuffers::FlatBufferBuilder;
use flatgeobuf::{ColumnType, GeometryType, HeaderBuilder};
use std::collections::HashSet;
use std::convert::TryInto;
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
//   description: string;          // Dataset description (intended for free form long text) //   metadata: string;             // Dataset metadata (intended to be application specific and suggested to be structured fx. JSON)
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

#[derive(Clone, Debug)]
pub struct ColSpec {
    pub name: String,
    pub type_: ColumnType,
}

fn col_specs(_features: &Vec<geojson::Feature>) -> Vec<ColSpec> {
    vec![ColSpec {
        name: "properties".to_string(),
        type_: ColumnType::Json,
    }]
}

pub fn write<'a>(features: &Vec<geojson::Feature>) -> (FlatBufferBuilder, Vec<ColSpec>) {
    let mut bldr = FlatBufferBuilder::new();
    // https://github.com/flatgeobuf/flatgeobuf/blob/master/src/fbs/header.fbs
    // https://github.com/flatgeobuf/flatgeobuf/blob/master/src/ts/generic/featurecollection.ts#L158-L182
    let name = bldr.create_string("L1");
    // let desc = bldr.create_string("");

    // let col_specs: Vec<ColSpec> = col_specs(features);
    let col_specs: Vec<ColSpec> = vec![];
    eprintln!("Columns for fgb file: {:?}", col_specs);
    let _cols_vec = columns::build(&mut bldr, &col_specs);

    let mut hb = HeaderBuilder::new(&mut bldr);
    // hb.add_description(desc);
    hb.add_features_count(features.len().try_into().unwrap()); // not sure when this would fail...i guess 128bit system?
    dbg!(geometry_type(features));
    hb.add_geometry_type(geometry_type(features));
    hb.add_index_node_size(0); // No Index? (following ts example)
    hb.add_name(name);
    let header = hb.finish();
    bldr.finish_size_prefixed(header, None);
    (bldr, col_specs)
}
