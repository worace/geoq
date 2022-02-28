use crate::geoq::{fgb::index, geojson::fvec};

use super::{columns, PropType};
use super::{hilbert::BoundedFeature, properties::col_type};
use super::{BBox, ColSpec};
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

fn schema<'a>(features: impl Iterator<Item = &'a geojson::Feature>) -> HashMap<String, PropType> {
    let mut schema = HashMap::<String, PropType>::new();
    for f in features {
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
                    if *current == PropType::JsonVal {
                        // Already using Json, most generic schema type, so leave as is
                        continue;
                    } else if jsont == PropType::Long && *current == PropType::Double {
                        // Already have Double and found a Long. Leave schema as is
                        // to "widen" from Long to double
                        continue;
                    } else {
                        // Widen from current specific type to more generic Json type
                        schema.insert(k.to_string(), PropType::JsonVal);
                    }
                }
            }
        }
    }
    schema
}

fn col_specs(features: &Vec<BoundedFeature>) -> Vec<ColSpec> {
    let schema = schema(features.iter().map(|f| &f.feature));
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

#[test]
fn test_schema_inference() {
    let gj = r#"{"type":"Feature","properties": {"name": "pizza", "age": 123},"geometry": {"type": "Point", "coordinates": [-118, 34]}}"#;
    let feats = fvec(gj);
    let sch = schema(feats.iter());
    assert_eq!(2, sch.len());
    assert_eq!(Some(&PropType::Long), sch.get("age"));
    assert_eq!(Some(&PropType::String), sch.get("name"));
}

#[test]
fn test_schema_inference_mixed() {
    let gj = r#"
      {"type": "FeatureCollection", "features": [
        {"type":"Feature","properties": {"name": "pizza", "n": "null"},"geometry": {"type": "Point", "coordinates": [-118, 34]}},
        {"type":"Feature","properties": {"foo": ["pizza"], "n": 123},"geometry": {"type": "Point", "coordinates": [-118, 34]}},
        {"type":"Feature","properties": {"foo": {"a":"b"}},"geometry": {"type": "Point", "coordinates": [-118, 34]}},
        {"type":"Feature","properties": {"name": 1.0},"geometry": {"type": "Point", "coordinates": [-118, 34]}},
        {"type":"Feature","properties": {"name": 1},"geometry": {"type": "Point", "coordinates": [-118, 34]}}
       ]}"#;
    let feats = fvec(gj);
    let sch = schema(feats.iter());
    assert_eq!(3, sch.len());
    assert_eq!(Some(&PropType::JsonVal), sch.get("name"));
    assert_eq!(Some(&PropType::JsonVal), sch.get("foo"));
    assert_eq!(Some(&PropType::JsonVal), sch.get("n"));
}
