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
    const point_gj: &str = r#"
     {"type": "Point", "coordinates": [-118, 34]}
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
        let (input, output) = roundtrip(point_gj);
        assert_eq!(input, output);
    }
}
