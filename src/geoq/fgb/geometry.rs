use std::convert::TryInto;

use flatbuffers::{FlatBufferBuilder, WIPOffset};
use flatgeobuf::GeometryType;

// Parsing geometry into FlatGeoBuf representation:
// https://github.com/flatgeobuf/flatgeobuf/blob/master/src/ts/generic/geometry.ts#L83-L112
#[derive(Debug)]
struct ParsedGeometry {
    xy: Vec<f64>,
    z: Option<Vec<f64>>,
    ends: Option<Vec<u32>>,
    parts: Option<Vec<ParsedGeometry>>,
    type_: GeometryType,
}

trait ParsedGeoJsonGeom {
    // fn xy(&self) -> Vec<f64>;
    fn parsed(&self) -> ParsedGeometry;
}

trait ParseGeom {
    fn xy(&self) -> Vec<f64>;
    fn z(&self) -> Option<Vec<f64>>;
    fn ends(&self) -> Option<Vec<u32>>;
    fn parts(&self) -> Option<Vec<ParsedGeometry>>;
}

impl ParseGeom for Vec<f64> {
    fn xy(&self) -> Vec<f64> {
        if self.len() < 2 {
            panic!("Invalid GeoJSON Point with missing x or y")
        }
        self[0..2].to_vec()
    }
    fn z(&self) -> Option<Vec<f64>> {
        if self.len() > 2 {
            Some(self[2..3].to_vec())
        } else {
            None
        }
    }
    fn ends(&self) -> Option<Vec<u32>> {
        None
    }
    fn parts(&self) -> Option<Vec<ParsedGeometry>> {
        None
    }
}

impl ParseGeom for Vec<Vec<f64>> {
    fn xy(&self) -> Vec<f64> {
        let mut xy: Vec<f64> = Vec::new();
        for p in self {
            xy.extend(p.xy());
        }
        xy
    }
    fn z(&self) -> Option<Vec<f64>> {
        let mut has_z = false;
        for coord in self {
            if coord.len() > 2 {
                has_z = true;
            }
        }
        if has_z {
            let mut z: Vec<f64> = Vec::new();
            for coord in self {
                z.push(*coord.get(2).unwrap_or(&0.0));
            }
            Some(z)
        } else {
            None
        }
    }
    fn ends(&self) -> Option<Vec<u32>> {
        None
    }
    fn parts(&self) -> Option<Vec<ParsedGeometry>> {
        None
    }
}

impl ParseGeom for Vec<Vec<Vec<f64>>> {
    fn xy(&self) -> Vec<f64> {
        let mut xy: Vec<f64> = Vec::new();
        for ring in self {
            for point in ring {
                xy.extend(point.xy());
            }
        }
        xy
    }
    fn z(&self) -> Option<Vec<f64>> {
        let mut has_z = false;
        for ring in self {
            for coord in ring {
                if coord.len() > 2 {
                    has_z = true;
                }
            }
        }
        if has_z {
            let mut z: Vec<f64> = Vec::new();
            for ring in self {
                for coord in ring {
                    z.push(*coord.get(2).unwrap_or(&0.0));
                }
            }
            Some(z)
        } else {
            None
        }
    }
    fn ends(&self) -> Option<Vec<u32>> {
        dbg!("get polygon ends");
        if self.len() > 1 {
            let mut ends: Vec<u32> = Vec::new();
            let mut num_coords = 0;
            for ring in self {
                num_coords += ring.len();
                // last_coord_start_idx += (ring.len() - 1) * 2;
                // "end" is index into flat coordinates for starting "X" of
                // coord pair where where each ring ends
                //     0 1    2 3     4 5    6 7    8 9
                // [ [[1,2], [3,4]] [[5,6], [7,8], [9,10]] ]
                //            End                   End.
                // ends: [2, 8] (coord idx 1 and coord idx 2, each doubled)
                // let end: u32 = ((num_coords - 1) * 2)
                // Or...?
                // end is just coord for _start_ of the next ring?
                let end: u32 = num_coords
                    .try_into()
                    .expect("Polygon end index must fit into u32");
                ends.push(end);
            }
            dbg!(&ends);
            Some(ends)
        } else {
            // No ends for single-ring polygon (following TS impl)
            None
        }
    }
    fn parts(&self) -> Option<Vec<ParsedGeometry>> {
        None
    }
}

impl ParsedGeoJsonGeom for geojson::Value {
    fn parsed(&self) -> ParsedGeometry {
        match self {
            geojson::Value::Point(coords) => ParsedGeometry {
                xy: coords.xy(),
                z: coords.z(),
                ends: None,
                parts: None,
                type_: GeometryType::Point,
            },
            geojson::Value::LineString(coords) => ParsedGeometry {
                xy: coords.xy(),
                z: coords.z(),
                ends: None,
                parts: None,
                type_: GeometryType::LineString,
            },
            geojson::Value::Polygon(coords) => ParsedGeometry {
                xy: coords.xy(),
                z: coords.z(),
                ends: coords.ends(),
                parts: None,
                type_: GeometryType::Polygon,
            },
            geojson::Value::MultiPoint(coords) => ParsedGeometry {
                xy: coords.xy(),
                z: coords.z(),
                ends: coords.ends(),
                parts: None,
                type_: GeometryType::MultiPoint,
            },
            geojson::Value::MultiLineString(coords) => ParsedGeometry {
                xy: coords.xy(),
                z: coords.z(),
                ends: coords.ends(),
                parts: None,
                type_: GeometryType::MultiLineString,
            },
            geojson::Value::MultiPolygon(coords) => {
                let parts = coords.iter().map(|poly_vec| ParsedGeometry {
                    xy: poly_vec.xy(),
                    z: poly_vec.z(),
                    ends: poly_vec.ends(),
                    parts: None,
                    type_: GeometryType::Polygon,
                });
                ParsedGeometry {
                    xy: vec![],
                    z: None,
                    ends: None,
                    parts: Some(parts.collect()),
                    type_: GeometryType::MultiPolygon,
                }
            }
            geojson::Value::GeometryCollection(geoms) => {
                let parts = geoms.iter().map(|geom| geom.value.parsed());
                ParsedGeometry {
                    xy: vec![],
                    z: None,
                    ends: None,
                    parts: Some(parts.collect()),
                    type_: GeometryType::GeometryCollection,
                }
            }
        }
    }
}

fn empty_parsed_geom() -> ParsedGeometry {
    ParsedGeometry {
        xy: Vec::new(),
        z: None,
        ends: None,
        parts: None,
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

fn _build<'a: 'b, 'b>(
    bldr: &'b mut FlatBufferBuilder<'a>,
    geom_components: &ParsedGeometry,
) -> WIPOffset<flatgeobuf::Geometry<'a>> {
    eprintln!("Parsed geom: {:?}", geom_components);

    let parts = geom_components.parts.as_ref().map(|geoms| {
        let g_offsets: Vec<WIPOffset<flatgeobuf::Geometry>> =
            geoms.iter().map(|g| _build(bldr, g)).collect();
        bldr.create_vector(&g_offsets[..])
    });

    let geom_args = flatgeobuf::GeometryArgs {
        xy: Some(bldr.create_vector(&geom_components.xy)),
        z: geom_components.z.as_ref().map(|z| bldr.create_vector(&z)),
        ends: geom_components
            .ends
            .as_ref()
            .map(|e| bldr.create_vector(&e)),
        type_: geom_components.type_,
        parts,
        ..Default::default()
    };

    let res = flatgeobuf::Geometry::create(bldr, &geom_args);
    res
}

pub fn build<'a: 'b, 'b>(
    bldr: &'b mut FlatBufferBuilder<'a>,
    f: &geojson::Feature,
) -> WIPOffset<flatgeobuf::Geometry<'a>> {
    let geom_components = f
        .geometry
        .as_ref()
        .map(|g| g.value.parsed())
        .unwrap_or(empty_parsed_geom());

    eprintln!("Parsed geom: {:?}", geom_components);
    _build(bldr, &geom_components)
}
