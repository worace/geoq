use super::header::ColSpec;
use flatgeobuf::ColumnType;
use serde_json::Map;
use std::convert::TryInto;

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
        if val_o.is_none() {
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
