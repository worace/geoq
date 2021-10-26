use super::header::ColSpec;
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
    fn _write_data(&self, idx: u16, vec: &mut Vec<u8>) -> () {
        if *self {
            vec.push(1);
        } else {
            vec.push(1);
        }
    }
}

pub fn feature_props(f: &geojson::Feature, _specs: &Vec<ColSpec>) -> Option<Vec<u8>> {
    return None;
    if f.properties.is_none() {
        return None;
    }
    let props: &Map<String, serde_json::Value> = f.properties.as_ref().unwrap();

    let mut bytes: Vec<u8> = Vec::new();
    let idx: u16 = 0;

    // Placeholder -- Single prop "properties" as stringified JSON
    let json = serde_json::to_string(&props).expect("Failed to serialize feature JSON properties");
    let json_bytes = json.as_bytes();
    let json_length: u32 = json_bytes
        .len()
        .try_into()
        .expect("Could not truncate String length to u32");
    // String encoding
    // index (u16)
    // bytes-length (u32)
    // bytes
    bytes.extend_from_slice(&idx.to_le_bytes());
    bytes.extend_from_slice(&json_length.to_le_bytes());
    bytes.extend_from_slice(&json_bytes);

    Some(bytes)
    // Placeholder

    // Real property writing would look sth like...
    // for c in specs {
    //     let prop = props.get(&c.name);
    //     if let Some(value) = prop {
    //         match c.type_ {
    //             ColumnType::Bool => match value {
    //                 serde_json::Value::Bool(b) => {
    //                     b.write(idx, &mut bytes);
    //                 }
    //                 _ => bytes.push(0),
    //             },
    //             ColumnType::Short => {
    //                 if value.is_i64() {
    //                     let int_val = value.as_i64().unwrap_or(0);
    //                     let short_val = i16::try_from(int_val).unwrap_or(0);
    //                     bytes.extend_from_slice(&short_val.to_le_bytes())
    //                 }
    //             }
    //             ColumnType::String => {}
    //             _ => (),
    //         }
    //     }
    //     idx += 1;
    // }
}
