use crate::geoq::fgb::index;

use super::columns;
use super::hilbert::BBox;
use super::hilbert::BoundedFeature;
use flatbuffers::FlatBufferBuilder;
use flatgeobuf::{ColumnType, GeometryType, HeaderArgs, HeaderBuilder};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::convert::TryInto;
use std::iter::Map;
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

fn geometry_type(features: &Vec<BoundedFeature>) -> GeometryType {
    let mut types = HashSet::new();
    let mut last_gtype = GeometryType::Unknown;
    for bf in features {
        let f = &bf.feature;
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

#[derive(PartialEq)]
enum PropType {
    Boolean,
    String,
    Long,
    Double,
    JsonVal,
}
// impl Eq for PropType {}

fn schema(features: &Vec<BoundedFeature>) -> HashMap<String, PropType> {
    let mut schema = HashMap::<String, PropType>::new();
    for bf in features {
        let f = &bf.feature;
        if f.properties.is_none() {
            continue;
        }
        for (k, v) in f.properties.as_ref().unwrap() {
            let jsont_o = match v {
                Value::Bool(_) => Some(PropType::Boolean),
                Value::String(_) => Some(PropType::String),
                Value::Number(num) => {
                    if num.is_f64() {
                        Some(PropType::Double)
                    } else if num.is_i64() {
                        Some(PropType::Long)
                    } else {
                        // Is this possible? I think is_f64 or is_i64 should cover all
                        None
                    }
                }
                Value::Array(_) => Some(PropType::JsonVal),
                Value::Object(_) => Some(PropType::JsonVal),
                Value::Null => None,
            };
            if jsont_o.is_none() {
                continue;
            }

            let jsont = jsont_o.unwrap();
            if !schema.contains_key(k) {
                schema.insert(k.to_string(), jsont);
            } else {
                let current = schema.get(k).unwrap();
                if *current == jsont {
                    continue;
                } else {
                    // schemas diverge for a key.
                    // 2 cases of widening:
                    // number: from Long -> Double
                    // any other (e.g. string vs array, string vs JSON):
                    // -> JsonVal
                    match jsont {
                        PropType::Long => {
                            if *current == PropType::Double {
                                // leave as is to "widen" from Long to double
                                continue;
                            }
                        }
                        _ => {
                            if *current == PropType::JsonVal {
                                continue;
                            } else {
                                schema.insert(k.to_string(), PropType::JsonVal);
                            }
                        }
                    }
                }
            }
        }
    }
    schema
}

fn col_type(prop_type: &PropType) -> ColumnType {
    match *prop_type {
        PropType::Boolean => ColumnType::Bool,
        PropType::Long => ColumnType::Long,
        PropType::Double => ColumnType::Double,
        PropType::String => ColumnType::String,
        PropType::JsonVal => ColumnType::Json,
    }
}

fn col_specs(features: &Vec<BoundedFeature>) -> Vec<ColSpec> {
    let schema = schema(features);
    schema
        .iter()
        .map(|(k, v)| ColSpec {
            name: k.to_string(),
            type_: col_type(v),
        })
        .collect()
}

pub fn write<'a>(
    features: &Vec<BoundedFeature>,
    bounds: &BBox,
) -> (FlatBufferBuilder<'a>, Vec<ColSpec>) {
    let mut bldr = FlatBufferBuilder::new();
    // https://github.com/flatgeobuf/flatgeobuf/blob/master/src/fbs/header.fbs
    // https://github.com/flatgeobuf/flatgeobuf/blob/master/src/ts/generic/featurecollection.ts#L158-L182
    let name = bldr.create_string("L1");
    // let desc = bldr.create_string("");

    let col_specs: Vec<ColSpec> = col_specs(features);
    let cols_vec = Some(columns::build(&mut bldr, &col_specs));

    let bounds_vec = bldr.create_vector(&bounds.to_vec());

    let args = HeaderArgs {
        name: Some(name),
        features_count: features.len().try_into().unwrap(), // not sure when this would fail...i guess 128bit system?
        geometry_type: geometry_type(features),
        index_node_size: index::NODE_SIZE,
        columns: cols_vec,
        envelope: Some(bounds_vec),
        ..Default::default()
    };

    let header = flatgeobuf::Header::create(&mut bldr, &args);
    bldr.finish_size_prefixed(header, None);
    (bldr, col_specs)
}
