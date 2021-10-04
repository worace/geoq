use crate::geoq::{entity::Entity, error::Error, reader::Reader};
use clap::ArgMatches;
use flatbuffers::{FlatBufferBuilder, ForwardsUOffset, UOffsetT, Vector, WIPOffset};
use flatgeobuf::{
    Column, ColumnArgs, ColumnBuilder, ColumnType, Feature, GeometryType, Header, HeaderBuilder,
};
use serde_json::Map;
use std::collections::HashSet;
use std::convert::{TryFrom, TryInto};
use std::io;

// Parsing geometry into FlatGeoBuf representation:
// https://github.com/flatgeobuf/flatgeobuf/blob/master/src/ts/generic/geometry.ts#L83-L112
struct ParsedGeometry {
    xy: Vec<f64>,
    z: Option<Vec<f64>>,
    ends: Option<Vec<usize>>,
    parts: Option<Vec<ParsedGeometry>>,
    type_: GeometryType,
}

trait ParsedGeoJsonGeom {
    // fn xy(&self) -> Vec<f64>;
    fn parsed(&self) -> ParsedGeometry;
}

trait XY {
    fn xy(&self) -> Vec<f64>;
}

impl XY for Vec<f64> {
    fn xy(&self) -> Vec<f64> {
        if self.len() < 2 {
            panic!("Invalid GeoJSON Point with missing x or y")
        }
        let xy = self[0..2].to_vec();
    }
}

impl ParsedGeoJsonGeom for geojson::Value {
    fn parsed(&self) -> ParsedGeometry {
        match *self {
            geojson::Value::Point(coords) => {
                if coords.len() < 2 {
                    panic!("Invalid GeoJSON Point with missing x or y")
                }
                let xy = coords[0..2].to_vec();
                let z = if coords.len() > 2 {
                    Some(coords[2..3].to_vec())
                } else {
                    None
                };
                ParsedGeometry {
                    xy,
                    z,
                    ends: None,
                    parts: None,
                    type_: GeometryType::Point,
                }
            }
            geojson::Value::LineString(coords) => {
                let mut xy: Vec<f64> = vec![];
                let has_z = false;
                for c in coords {
                    xy.push(c[0]);
                    xy.push(c[1]);
                    if c.len() > 2 {
                        has_z = true;
                    }
                }

                let z = if has_z {
                    let mut z: Vec<f64> = vec![];
                    for c in coords {
                        if c.len() > 2 {
                            z.push(c[2]);
                        } else {
                            z.push(0.0);
                        }
                    }
                    Some(z)
                } else {
                    None
                };

                ParsedGeometry {
                    xy,
                    z,
                    ends: None,
                    parts: None,
                    type_: GeometryType::Unknown,
                }
            }
            _ => ParsedGeometry {
                xy: Vec::new(),
                z: None,
                ends: None,
                parts: None,
                type_: GeometryType::Unknown,
            },
        }
    }
}

fn parse_geom(g: &geojson::Value) -> ParsedGeometry {
    ParsedGeometry {
        xy: Vec::new(),
        z: Vec::new(),
        ends: Vec::new(),
        parts: Vec::new(),
        type_: GeometryType::Unknown,
    }
}

// https://www.notion.so/worace/Flatgeobuf-4c2eb8ea1475419991863f36bd2fa355

// table Geometry {
//   ends: [uint];          // Array of end index in flat coordinates per geometry part
//   xy: [double];          // Flat x and y coordinate array (flat pairs)
//   z: [double];           // Flat z height array
//   m: [double];           // Flat m measurement array
//   t: [double];           // Flat t geodetic decimal year time array
//   tm: [ulong];           // Flat tm time nanosecond measurement array
//   type: GeometryType;    // Type of geometry (only relevant for elements in heterogeneous collection types)
//   parts: [Geometry];     // Array of parts (for heterogeneous collection types)
// }

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
    // https://github.com/flatgeobuf/flatgeobuf/blob/master/src/ts/generic/feature.ts#L47-L143
    // flatgeobuf::GeometryOffset

    // Q: should this repeat all columns for the schema, or only the ones that apply to this feature?
    // Copy-Pastad code from header section
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
        })
        .collect();
    let cols_vec = bldr.create_vector(&cols[..]);

    let props = feature_props(f, col_specs).map(|bytes| bldr.create_vector(&bytes[..]));

    // Geometry serialization
    // https://github.com/flatgeobuf/flatgeobuf/blob/master/src/ts/generic/geometry.ts#L37-L64

    let args = flatgeobuf::FeatureArgs {
        columns: Some(cols_vec),
        geometry: None,
        properties: props,
    };
    let offset = flatgeobuf::Feature::create(bldr, &args);

    bldr.finish(offset, None);
}

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

fn feature_props(f: &geojson::Feature, _specs: &Vec<ColSpec>) -> Option<Vec<u8>> {
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
) -> Vec<ColSpec> {
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
    col_specs
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
    let col_specs = write_header(&mut header_builder, &input_features);
    // Header is now done, use header_builder.finished_data() to access &[u8]

    let mut feature_builders: Vec<FlatBufferBuilder> = Vec::new();

    for f in input_features {
        let mut builder = FlatBufferBuilder::new();
        write_feature(&mut builder, &col_specs, &f);
        feature_builders.push(builder);
    }

    // write_fgb_feature(&mut builder, &e);
    Ok(())
}

pub fn run(_fgb: &ArgMatches) -> Result<(), Error> {
    write()
}
