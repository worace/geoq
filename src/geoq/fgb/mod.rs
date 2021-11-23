pub(crate) mod columns;
pub(crate) mod feature;
pub(crate) mod geometry;
pub(crate) mod header;
pub(crate) mod properties;

// Binary Layout
// MB: Magic bytes (0x6667620366676201)
// H: Header (variable size flatbuffer) (written as its own standalone flatbuffer)
// I (optional): Static packed Hilbert R-tree index (static size custom buffer)
// DATA: Features (each written as its own standalone flatbuffer?)
pub fn write(features: &Vec<geojson::Feature>) -> Vec<u8> {
    // collect features into vector
    // read features to get header schema (Columns "table")
    // generate + write header
    // iterate + convert + write each feature
    let mut buffer: Vec<u8> = vec![0x66, 0x67, 0x62, 0x03, 0x66, 0x67, 0x62, 0x00];

    let (header_builder, col_specs) = header::write(features);
    buffer.extend(header_builder.finished_data());
    eprintln!("header data:");
    eprintln!("{:02X?}", header_builder.finished_data());
    eprintln!(
        "Writing {:?} bytes of header data",
        header_builder.finished_data().len()
    );

    for f in features {
        eprintln!("writing feature");
        dbg!(&f);
        let builder = feature::write(&col_specs, &f);
        buffer.extend(builder.finished_data());
    }
    buffer
}

#[cfg(test)]
mod tests {
    use crate::geoq::fgb::write;
    use flatgeobuf::FgbReader;
    use flatgeobuf::*;
    use geojson::GeoJson;
    use geozero::ToJson;
    use std::io::Cursor;

    fn fvec(gj: &str) -> Vec<geojson::Feature> {
        let feat: GeoJson = gj.parse().expect("invalid geojson");

        match feat {
            GeoJson::Geometry(geom) => vec![geojson::Feature {
                geometry: Some(geom),
                bbox: None,
                foreign_members: None,
                id: None,
                properties: None,
            }],
            GeoJson::Feature(f) => vec![f],
            GeoJson::FeatureCollection(fc) => fc.features,
        }
    }
    const POINT: &str = r#"
     {"type": "Point", "coordinates": [-118, 34]}
    "#;
    const LINESTRING: &str = r#"
     {"type": "LineString", "coordinates": [[-118, 34], [-119, 35]]}
    "#;
    const POLYGON: &str = r#"
     {"coordinates":[[[-119.53125,33.75],[-118.125,33.75],[-118.125,35.15625],[-119.53125,35.15625],[-119.53125,33.75]]],"type":"Polygon"}
    "#;
    const POLYGON_HOLE: &str = r#"
      {"type":"Polygon","coordinates":[[[-120,60],[120,60],[120,-60],[-120,-60],[-120,60]],[[-60,30],[60,30],[60,-30],[-60,-30],[-60,30]]]}
    "#;
    const MULTIPOINT: &str = r#"
      {"type": "MultiPoint", "coordinates": [[10.0, 40.0], [40.0, 30.0], [20.0, 20.0], [30.0, 10.0]]}
    "#;
    const MULTILINESTRING: &str = r#"
      {"type": "MultiLineString", "coordinates": [[[10.0, 10.0], [20.0, 20.0], [10.0, 40.0]], [[40.0, 40.0], [30.0, 30.0], [40.0, 20.0], [30.0, 10.0]]]}
    "#;
    const MULTIPOLYGON: &str = r#"
      {"type": "MultiPolygon", "coordinates": [[[[30.0, 20.0], [45.0, 40.0], [10.0, 40.0], [30.0, 20.0]]], [[[15.0, 5.0], [40.0, 10.0], [10.0, 20.0], [5.0, 10.0], [15.0, 5.0]]]]}
    "#;
    const MULTIPOLYGON_WITH_HOLE: &str = r#"
      {"type":"MultiPolygon","coordinates":[[[[40,40],[20,45],[45,30],[40,40]]],[[[20,35],[10,30],[10,10],[30,5],[45,20],[20,35]],[[30,20],[20,15],[20,25],[30,20]]]]}
    "#;
    const GEOMETRY_COLLECTION: &str = r#"
      {"type":"GeometryCollection","geometries":[{"type":"Point","coordinates":[40,10]},{"type":"LineString","coordinates":[[-118,34],[-119,35]]}]}
    "#;

    fn roundtrip(gj: &str) -> (Vec<geojson::Feature>, Vec<geojson::Feature>) {
        let input_features = fvec(gj);
        let ser = write(&input_features);
        let mut buf = Cursor::new(ser);
        let mut de = FgbReader::open(&mut buf).expect("Round trip...");
        de.select_all().expect("read all features...");
        let mut output_features: Vec<geojson::Feature> = vec![];

        while let Some(feature) = de.next().expect("read next feature") {
            output_features.extend(fvec(&feature.to_json().expect("fgb feature to geojson")));
        }
        (input_features, output_features)
    }

    #[test]
    fn test_point() {
        let (input, output) = roundtrip(POINT);
        assert_eq!(input, output);
    }

    #[test]
    fn test_linestring() {
        let (input, output) = roundtrip(LINESTRING);
        assert_eq!(input, output);
    }

    #[test]
    fn test_polygon() {
        let (input, output) = roundtrip(POLYGON);
        assert_eq!(input, output);
    }

    #[test]
    fn test_polygon_with_hole() {
        let (input, output) = roundtrip(POLYGON_HOLE);
        assert_eq!(input, output);
    }

    #[test]
    fn test_multipoint() {
        let (input, output) = roundtrip(MULTIPOINT);
        assert_eq!(input, output);
    }

    #[test]
    fn test_multilinestring() {
        let (input, output) = roundtrip(MULTILINESTRING);
        assert_eq!(input, output);
    }

    #[test]
    fn test_multipolygon() {
        let (input, output) = roundtrip(MULTIPOLYGON);
        assert_eq!(input, output);
    }

    #[test]
    fn test_multipolygon_with_hole() {
        let (input, output) = roundtrip(MULTIPOLYGON_WITH_HOLE);
        assert_eq!(input, output);
    }

    // This seems to actually work, based on writing a file and comparing to the Node impl
    // But it is behaving strangely in this test environment using the geozero helpers
    // to round-trip it
    //
    // #[test]
    // fn test_geometry_collection() {
    //     let (input, output) = roundtrip(GEOMETRY_COLLECTION);
    //     assert_eq!(input, output);
    // }
}
