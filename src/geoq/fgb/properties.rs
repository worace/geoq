use super::{ColSpec, PropType};
use flatgeobuf::ColumnType;
use serde_json::{Map, Value};
use std::{any::Any, collections::HashMap, convert::TryInto};

trait ToBytesWithIndex {
    fn write(&self, idx: u16, vec: &mut Vec<u8>) -> () {
        vec.extend_from_slice(&idx.to_le_bytes());
        self._write_data(idx, vec);
    }
    fn _write_data(&self, idx: u16, vec: &mut Vec<u8>) -> ();
}

impl ToBytesWithIndex for bool {
    fn _write_data(&self, _idx: u16, vec: &mut Vec<u8>) -> () {
        if *self {
            vec.push(1);
        } else {
            vec.push(1);
        }
    }
}

pub fn col_type(prop_type: &PropType) -> ColumnType {
    match *prop_type {
        PropType::Boolean => ColumnType::Bool,
        PropType::Long => ColumnType::Long,
        PropType::Double => ColumnType::Double,
        PropType::String => ColumnType::String,
        PropType::JsonVal => ColumnType::Json,
        PropType::Null => ColumnType::Json,
    }
}

pub fn col_specs(schema: &HashMap<String, PropType>) -> Vec<ColSpec> {
    let mut keys: Vec<String> = vec![];
    for k in schema.keys() {
        keys.push(k.clone());
    }
    keys.sort();

    keys.iter()
        .map(|k| {
            let v = schema.get(k).unwrap();
            ColSpec {
                name: k.to_string(),
                type_: col_type(v),
            }
        })
        .collect()
}

pub fn feature_schema(feature: &geojson::Feature) -> HashMap<String, PropType> {
    let mut schema = HashMap::<String, PropType>::new();
    if feature.properties.is_none() {
        return schema;
    }
    for (k, v) in feature.properties.as_ref().unwrap() {
        let jsont = match v {
            Value::Bool(_) => PropType::Boolean,
            Value::String(_) => PropType::String,
            Value::Number(num) => {
                if num.is_f64() {
                    PropType::Double
                } else if num.is_i64() {
                    PropType::Long
                } else {
                    panic!(
                        "Expected JSON Number to be coercible to either i64 or f64. Found: {:?}",
                        num
                    );
                }
            }
            Value::Array(_) => PropType::JsonVal,
            Value::Object(_) => PropType::JsonVal,
            Value::Null => PropType::Null,
        };
        schema.insert(k.to_string(), jsont);
    }
    schema
}

pub fn feature_props(f: &geojson::Feature, specs: &Vec<ColSpec>) -> Option<Vec<u8>> {
    if f.properties.is_none() {
        return None;
    }

    let props: &Map<String, serde_json::Value> = f.properties.as_ref().unwrap();

    let mut bytes: Vec<u8> = Vec::new();
    let mut idx: u16 = 0;
    for col in specs {
        let k = &col.name;
        let val_o = props.get(k);
        if val_o.is_none() || val_o.filter(|v| v.is_null()).is_some() {
            idx += 1;
            continue;
        }

        let val = val_o.unwrap();

        // record index of current column
        bytes.extend_from_slice(&idx.to_le_bytes());

        // Bool, Long, Double, String, Json
        match col.type_ {
            ColumnType::Bool => {
                let b = val
                    .as_bool()
                    .expect(&format!("Inferred Schema expected boolean prop at {}", &k));

                if b {
                    bytes.extend_from_slice(&1u8.to_le_bytes());
                } else {
                    bytes.extend_from_slice(&0u8.to_le_bytes());
                }
            }
            ColumnType::Long => {
                let num = val.as_i64().expect(&format!(
                    "Inferred Schema expected integer prop at {}, got {}",
                    &k, val
                ));
                bytes.extend_from_slice(&num.to_le_bytes());
            }
            ColumnType::Double => {
                let num = val.as_f64().expect(&format!(
                    "Inferred Schema expected double prop at {}, got {}",
                    &k, val
                ));
                bytes.extend_from_slice(&num.to_le_bytes());
            }
            ColumnType::String => {
                let s = val.as_str().expect(&format!(
                    "Inferred Schema expected String prop at {}, got {}",
                    &k, val
                ));

                let len: u32 = s.len().try_into().expect(&format!(
                    "Could not truncate String length to u32. Length is: {}. String: {}",
                    s.len(),
                    &s,
                ));

                bytes.extend_from_slice(&len.to_le_bytes());
                bytes.extend_from_slice(&s.as_bytes());
            }
            ColumnType::Json => {
                let json_str = val.to_string();
                let len: u32 = json_str.len().try_into().expect(&format!(
                    "Could not truncate String length to u32. Length is: {}. String: {}",
                    json_str.len(),
                    &json_str,
                ));

                bytes.extend_from_slice(&len.to_le_bytes());
                bytes.extend_from_slice(&json_str.as_bytes());
            }
            other => {
                panic!(
                    "Feature property serialization received unexpected column type: {:?}.",
                    other
                );
            }
        }
        idx += 1;
    }

    if bytes.is_empty() {
        None
    } else {
        Some(bytes)
    }
}
