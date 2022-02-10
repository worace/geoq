use crate::geoq::fgb::hilbert::IndexNode;

pub(crate) mod columns;
pub(crate) mod feature;
pub(crate) mod geometry;
pub(crate) mod header;
pub(crate) mod hilbert;
pub(crate) mod index;
pub(crate) mod properties;

// TODO
// * [x] Add envelope generation and record in header field
// * [x] Add hilbert sort
// * [x] Add packed rtree index
// * [ ] Support streaming write (2-pass) - Is this possible with hilbert sort?
//       - needs external merge sort with on-disk buffers?
// * [ ] Implement paged slippy-map UI for TS

// Hilbert Sort / Index
// 1. [X] Calc bboxes for all nodes (NodeItem:)
//        min_x,min_y,max_x,max_y -> (Feature, BBox)
// 2. [X] Get dataset (ds) "extent" -- total bbox of dataset
//        (fold 'expand' over feature bboxes)
// 3. [X]  sort by hilbert bboxes
//         sort_by { |feat, bbox| hilbert_bbox(bbox, ds_extent) } (hilbert_bbox(feat_bbox, max_val, ds_bbox) -> u32)
// 4. Write features to buffer...
//   - write feature
//   - record byte offset
//   - use (bbox, byte_offset) pairs for building index
// 5.

// Binary Layout
// MB: Magic bytes (0x6667620366676201)
// H: Header (variable size flatbuffer) (written as its own standalone flatbuffer)
// I (optional): Static packed Hilbert R-tree index (static size custom buffer)
// DATA: Features (each written as its own standalone flatbuffer?)
pub fn write(features: Vec<geojson::Feature>) -> Vec<u8> {
    // collect features into vector
    // read features to get header schema (Columns "table")
    // generate + write header
    // iterate + convert + write each feature
    let mut buffer: Vec<u8> = vec![0x66, 0x67, 0x62, 0x03, 0x66, 0x67, 0x62, 0x00];
    let mut features_temp_buffer: Vec<u8> = vec![];

    let (bounded_sorted_features, dataset_bounds) = hilbert::sort_with_extent(features);

    let (header_builder, col_specs) = header::write(&bounded_sorted_features, &dataset_bounds);
    buffer.extend(header_builder.finished_data());
    eprintln!("header data:");
    eprintln!("{:02X?}", header_builder.finished_data());
    eprintln!(
        "Writing {:?} bytes of header data",
        header_builder.finished_data().len()
    );

    // Writing:
    // Buffer A (Main, could be file):
    // Buffer B (temp features, tmpfile?)
    //   1. Sort features, calc extent + header
    //   2. Write header to A
    //   3. Write features to Buffer B, record byte offsets + BBoxes
    //   4. Build RTREE using byte offsets + BBoxes
    //   5. Write RTree bytes to A
    //   6. Copy features tempfile data from B to A

    // TODO: write features to tempfile, so it can be copied to end of buffer
    let mut offsets_for_index: Vec<IndexNode> = vec![];
    for f in bounded_sorted_features {
        let feature_offset = features_temp_buffer.len();
        offsets_for_index.push(IndexNode {
            offset: feature_offset,
            bbox: f.bbox,
        });
        let builder = feature::write(&col_specs, &f.feature);
        features_temp_buffer.extend(builder.finished_data());
    }
    let (_layout, flattened_tree) =
        index::build_flattened_tree(offsets_for_index, &dataset_bounds, index::NODE_SIZE);
    let index_bytes = index::serialize(flattened_tree);
    buffer.extend(index_bytes);
    buffer.extend(features_temp_buffer);
    buffer
}

#[cfg(test)]
mod tests {
    use crate::geoq::{
        fgb::{
            hilbert::IndexNode,
            index::{self, RTreeIndexMeta},
            write,
        },
        reader::Reader,
    };
    use flatgeobuf::FgbReader;
    use flatgeobuf::*;
    use geojson::GeoJson;
    use std::io::{Cursor, Read, Seek};

    fn fvec(gj: &str) -> Vec<geojson::Feature> {
        use serde_json::json;
        let j: serde_json::Value = gj.parse().expect("couldn't parse json");
        let feat: GeoJson = gj.parse().expect("invalid geojson");

        match feat {
            GeoJson::Geometry(geom) => vec![geojson::Feature {
                geometry: Some(geom),
                bbox: None,
                foreign_members: None,
                id: None,
                properties: Some(serde_json::Map::new()),
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
    const POINT_PROPS: &str = r#"
      {"type":"Feature","properties": {"name": "pizza"},"geometry": {"type": "Point", "coordinates": [-118, 34]}}
    "#;

    const MULTI_SCHEMA: &str = r#"
      {"type": "FeatureCollection", "features":[
        {"type":"Feature","properties": {"name": "pizza", "age": 123},"geometry": {"type": "Point", "coordinates": [-118, 34]}},
        {"type":"Feature","properties": {"name": "pizza", "age": 456},"geometry": {"type": "Point", "coordinates": [-118, 34]}}
       ]}
    "#;

    fn roundtrip(gj: &str) -> (Vec<geojson::Feature>, Vec<geojson::Feature>) {
        use geozero::ProcessToJson;
        // use geozero::ToJson;

        let input_features = fvec(gj);
        let ser = write(input_features.clone());
        let mut buf: Cursor<Vec<u8>> = Cursor::new(ser);
        let mut de = FgbReader::open(&mut buf).expect("Round trip...");
        de.select_all().expect("read all features...");

        let mut output_features: Vec<geojson::Feature> = vec![];
        let deserialized_geojson: String = de.to_json().unwrap();
        output_features = fvec(&deserialized_geojson);

        // while let Some(feature) = de.next().expect("read next feature") {
        //     let props = feature.properties();
        //     dbg!(props.unwrap());
        //     dbg!(&feature.to_json());
        //     output_features.extend(fvec(&feature.to_json().expect("fgb feature to geojson")));
        // }
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

    // #[test]
    // fn test_samples() {
    //     let points = std::fs::read_to_string("./samples/points.geojson").unwrap();
    //     let (input, output) = roundtrip(&points);

    //     for (i, o) in input.iter().zip(output.iter()) {
    //         assert_eq!(i, o);
    //     }
    // }

    #[test]
    fn test_point_props() {
        let (input, output) = roundtrip(POINT_PROPS);
        assert_eq!(input, output);
    }

    #[test]
    fn test_multi_schema() {
        let (input, output) = roundtrip(MULTI_SCHEMA);
        assert_eq!(input, output);
    }

    #[test]
    fn test_header() {
        let input_features = fvec(POINT_PROPS);
        let ser = write(input_features.clone());
        let mut buf = Cursor::new(ser);
        let res = FgbReader::open(&mut buf).expect("Round trip...");

        let bounds: Vec<f64> = res
            .header()
            .envelope()
            .expect("Header should have an envelope populated")
            .iter()
            .collect();
        dbg!(&bounds);
        assert_eq!(bounds, vec![-118.0, 34.0, -118.0, 34.0]);
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

    use std::fs::File;
    use std::io::BufReader;
    use std::io::Write;
    use tempfile::tempfile;
    use tempfile::NamedTempFile;
    #[test]
    fn test_countries_dataset() {
        use geozero::ProcessToJson;

        let input_file = File::open("./tests/resources/countries.geojson").unwrap();
        let mut input_buffer = BufReader::new(input_file);

        let mut features: Vec<geojson::Feature> = vec![];
        let reader = Reader::new(&mut input_buffer);

        for e_res in reader {
            if let Ok(entity) = e_res {
                features.push(entity.geojson_feature())
            }
        }
        assert_eq!(179, features.len());

        let buffer = write(features);
        // let mut output_file = NamedTempFile::new().unwrap();
        let path = "/Users/horace/data/fgb_samples/geoq_countries.fgb";
        let mut output_file = File::create(path).unwrap();
        dbg!(&output_file);
        output_file.write(&buffer).unwrap();

        // let mut comp_file = output_file.reopen().unwrap();
        let mut comp_file = File::open(path).unwrap();
        let mut ref_impl = FgbReader::open(&mut comp_file).unwrap();

        ref_impl.select_bbox(8.8, 47.2, 9.5, 55.3).unwrap();

        let mut output_features: Vec<geojson::Feature> = vec![];
        let deserialized_geojson: String = ref_impl.to_json().unwrap();
        output_features = fvec(&deserialized_geojson);

        assert_eq!(output_features.len(), 6);
        for f in output_features {
            eprintln!("{:?}", f.properties);
        }
        // dbg!(output_features);

        // read using geozero
    }

    #[test]
    #[ignore]
    fn test_reading_large_data() {
        use geozero::ProcessToJson;

        let path = "/Users/horace/data/fgb_samples/alabama.fgb";

        // let mut comp_file = output_file.reopen().unwrap();
        let mut comp_file = File::open(path).unwrap();
        let mut ref_impl = FgbReader::open(&mut comp_file).unwrap();

        ref_impl
            .select_bbox(-86.286733, 32.367310, -86.276207, 32.373978)
            .unwrap();

        let mut output_features: Vec<geojson::Feature> = vec![];
        let deserialized_geojson: String = ref_impl.to_json().unwrap();
        output_features = fvec(&deserialized_geojson);

        // actual: 79 -- matches JS-based impl and rust-based readers
        // so index is probably messed up somehow
        // assert_eq!(output_features.len(), 389);
        // for f in output_features {
        //     eprintln!("{:?}", f.properties);
        // }
        // dbg!(output_features);

        // read using geozero
    }

    #[test]
    #[ignore]
    fn test_writing_with_geozero() {
        use flatgeobuf::*;
        use geozero::geojson::GeoJsonReader;
        use geozero::GeozeroDatasource;
        use std::fs::File;
        use std::io::{BufReader, BufWriter};

        let mut fgb = FgbWriter::create("alabama", GeometryType::Polygon, |_, _| {}).unwrap();
        let mut fin =
            BufReader::new(File::open("/Users/horace/Downloads/Alabama.geojson").unwrap());
        let mut reader = GeoJsonReader(&mut fin);
        reader.process(&mut fgb).unwrap();
        let mut fout = BufWriter::new(
            File::create("/Users/horace/data/fgb_samples/geozero_alabama.fgb").unwrap(),
        );
        fgb.write(&mut fout).unwrap();
    }

    #[test]
    fn test_index_comps() {
        use flatgeobuf::*;
        use geozero::geojson::GeoJsonReader;
        use geozero::GeozeroDatasource;
        use std::fs::File;
        use std::io::{BufReader, BufWriter};

        let source_path = "/Users/horace/data/geojson_samples/alabama500.geojson";
        let geoq_output_path = "/tmp/alabama20k_geoq.fgb";
        let geozero_output_path = "/tmp/alabama20k_geozero.fgb";
        ////////////////
        // Write geoq
        eprintln!("write geoq");
        let input_file = File::open(source_path).unwrap();
        let mut input_buffer = BufReader::new(input_file);

        let mut features: Vec<geojson::Feature> = vec![];
        let reader = Reader::new(&mut input_buffer);

        for e_res in reader {
            if let Ok(entity) = e_res {
                features.push(entity.geojson_feature())
            }
        }
        assert_eq!(500, features.len());

        let buffer = write(features);
        // let mut output_file = NamedTempFile::new().unwrap();
        let mut output_file = File::create(geoq_output_path).unwrap();
        dbg!(&output_file);
        output_file.write(&buffer).unwrap();
        ///////////////

        ////////////////
        // Write GeoZero
        // eprintln!("write geozero");
        // let mut fgb = FgbWriter::create("alabama", GeometryType::Polygon, |_, _| {}).unwrap();
        // let mut fin = BufReader::new(File::open(source_path).unwrap());
        // let mut reader = GeoJsonReader(&mut fin);
        // reader.process(&mut fgb).unwrap();
        // let mut fout = BufWriter::new(File::create(geozero_output_path).unwrap());
        // fgb.write(&mut fout).unwrap();
        //////////////////

        ///////////////
        // Open and compare index nodes
        let mut geoq_file = File::open(geoq_output_path).unwrap();
        let mut ref_impl = FgbReader::open(&mut geoq_file).unwrap();
        let geoq_bytes: Vec<u8> = std::fs::read(geoq_output_path).unwrap();
        let geozero_bytes = std::fs::read(geozero_output_path).unwrap();

        eprintln!(
            "geoq bytes {}, geozero bytes {}",
            geoq_bytes.len(),
            geozero_bytes.len(),
        );

        let (geoq_header, gq_tree, geoq_tree_bytes) = header_and_index_nodes(&geoq_bytes);
        let (gz_header, gz_tree, gz_tree_bytes) = header_and_index_nodes(&geozero_bytes);
        dbg!(&geoq_header);
        dbg!(&gz_header);
        dbg!(&gq_tree);
        dbg!(&gz_tree);

        for i in (0..5) {
            let gq_node = IndexNode::from_bytes(&geoq_tree_bytes[i * 40..(i + 1) * 40]).unwrap();
            let gz_node = IndexNode::from_bytes(&gz_tree_bytes[i * 40..(i + 1) * 40]).unwrap();
            eprintln!("gq: {:?}", gq_node);
            eprintln!("gz: {:?}", gz_node);
        }

        eprintln!("Done...");
        // flatgeobuf::Header, u8
        fn header_and_index_nodes(bytes: &Vec<u8>) -> (Header, RTreeIndexMeta, Vec<u8>) {
            let num_features = 500;
            let header_size: u8 = bytes[8]; // assuming these fit in 1 byte
            eprintln!("leading 8: {:?}", bytes[0..8].to_vec());
            eprintln!("header size: {:?}", header_size);
            let header_bytes = &bytes[13..];
            let header = flatgeobuf::size_prefixed_root_as_header(header_bytes).unwrap();
            let node_size = header.index_node_size();
            let tree_meta = index::calculate_level_bounds(num_features, node_size);
            let tree_start: usize = 8 + 4 + header_size as usize;
            let tree_size: usize = tree_meta.num_nodes * 5;

            (
                header,
                tree_meta,
                bytes[tree_start..tree_start + tree_size].to_vec(),
            )
        }
    }
}

// bbox
//   -85.0316047668457,
//   32.45169343044352
// ],
// [
//   -84.99555587768553,
//   32.45169343044352
// ],
// [
//   -84.99555587768553,
//   32.48037024365968
// ],
// [
//   -85.0316047668457,
//   32.48037024365968
// ],
// [
//   -85.0316047668457,
//   32.45169343044352
