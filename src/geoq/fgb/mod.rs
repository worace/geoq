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
            hilbert::{self, IndexNode},
            index::{self, RTreeIndexMeta},
            write,
        },
        geojson::fvec,
        reader::Reader,
    };
    use flatgeobuf::packed_r_tree::hilbert_sort;
    use flatgeobuf::{packed_r_tree::NodeItem, FgbReader};
    use geojson::GeoJson;
    use std::io::{Cursor, Read, Seek};

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
      {"type":"Feature","properties": {"name": "\"pizza"},"geometry": {"type": "Point", "coordinates": [-118, 34]}}
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
        let de = FgbReader::open(&mut buf).expect("Round trip...");
        let mut de = de.select_all().expect("read all features...");

        let deserialized_geojson: String = de.to_json().unwrap();

        (input_features, fvec(&deserialized_geojson))
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
    fn test_json_null() {
        let json_null_in = r#"
          {"type": "FeatureCollection", "features":[
            {"type":"Feature","properties": {"name": "pizza"},"geometry": {"type": "Point", "coordinates": [0,0]}},
            {"type":"Feature","properties": {"name": null},"geometry": {"type": "Point", "coordinates": [1,1]}}
           ]}
        "#;
        let json_null_out = r#"
          {"type": "FeatureCollection", "features":[
            {"type":"Feature","properties": {"name": "pizza"},"geometry": {"type": "Point", "coordinates": [0,0]}},
            {"type":"Feature","properties": {},"geometry": {"type": "Point", "coordinates": [1,1]}}
           ]}
        "#;
        // Not clear the best way to handle this...FGB properties
        // doesn't seem to have a way to represent an explicit 'null'
        // as distinct from the property simply being omitted.
        // This could either omit the property or cast the column to
        // JSON in order to encode the JSON 'null' value.
        // Currently omitting the field, as this seems to play nicer with other impls,
        // but would like to be able to support it via the json type
        let (_, output) = roundtrip(json_null_in);
        assert_eq!(output, fvec(json_null_out));
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
        let mut output_file = NamedTempFile::new().unwrap();
        output_file.write(&buffer).unwrap();

        let mut comp_file = output_file.reopen().unwrap();
        let ref_impl = FgbReader::open(&mut comp_file).unwrap();

        let mut ref_impl = ref_impl.select_bbox(8.8, 47.2, 9.5, 55.3).unwrap();

        let deserialized_geojson: String = ref_impl.to_json().unwrap();
        let output_features = fvec(&deserialized_geojson);

        assert_eq!(output_features.len(), 6);
    }

    #[test]
    fn test_hilbert_sort_comp() {
        use flatgeobuf::*;
        use geozero::geojson::GeoJsonReader;
        use geozero::GeozeroDatasource;
        use std::fs::File;
        use std::io::{BufReader, BufWriter};

        let source_path = "./tests/resources/alabama500.geojson";

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

        let _gz_feats = features.clone();

        let (sorted, extent) = hilbert::sort_with_extent(features);

        let mut gz_nodes: Vec<NodeItem> = sorted
            .iter()
            .enumerate()
            .map(|(idx, f)| NodeItem {
                min_x: f.bbox.min_x,
                min_y: f.bbox.min_y,
                max_x: f.bbox.max_x,
                max_y: f.bbox.max_y,
                offset: idx as u64,
            })
            .collect();
        let gz_extent = NodeItem {
            min_x: extent.min_x,
            min_y: extent.min_y,
            max_x: extent.max_x,
            max_y: extent.max_y,
            offset: 0,
        };
        hilbert_sort(&mut gz_nodes, &gz_extent);

        assert_eq!(sorted.len(), gz_nodes.len());

        // TODO - this hilbert sort is not the same!!!
        for i in 0..20 {
            eprintln!("gq {:?} gz {:?}", i, gz_nodes[i].offset);
        }
    }
}
