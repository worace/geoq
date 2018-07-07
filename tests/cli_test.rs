extern crate assert_cli;
// extern crate geoq;
use assert_cli::Assert;

#[test]
fn it_finds_types() {
    let input = "12,34
9q5
{\"type\":\"Point\",\"coordinates\":[125.6, 10.1]}
LINESTRING (30 10, 10 30, 40 40)
pizza
";

    let output = "LatLon(12,34)
Geohash(9q5)
GeoJSON({\"type\":\"Point\",\"coordinates\":[125.6, 10.1]})
WKT(LINESTRING (30 10, 10 30, 40 40))
Unknown(pizza)
";
    println!("{}", output);
    Assert::main_binary()
        .with_args(&["type"])
        .stdin(input)
        .stdout()
        .contains(output)
        .unwrap();
}

#[test]
fn it_outputs_wkt() {
    let input = r#"12,34
12	34
9q5
LINESTRING (30 10, 10 30, 40 40)
{"type":"Point","coordinates":[125.6, 10.1]}
{"type":"Feature","properties":{"a": "b"},"geometry":{"type":"Point","coordinates":[125.6, 10.1]}}
{"type":"FeatureCollection","features":[{"type":"Feature","properties":{},"geometry":{"type":"Point","coordinates":[34.0,12.0]}},{"type":"Feature","properties":{},"geometry":{"type":"Point","coordinates":[78.0,56.0]}}]}
"#;

    let output = r#"POINT(34 12)
POINT(34 12)
POLYGON((-119.53125 33.75,-118.125 33.75,-118.125 35.15625,-119.53125 35.15625,-119.53125 33.75))
LINESTRING(30 10,10 30,40 40)
POINT(125.6 10.1)
POINT(125.6 10.1)
POINT(34 12)
POINT(78 56)
"#;
    Assert::main_binary()
        .with_args(&["wkt"])
        .stdin(input)
        .stdout()
        .is(output)
        .unwrap();
}

#[test]
fn outputs_geojson_geoms() {
    let input = r#"12,34
12	34
9q5
LINESTRING (30 10, 10 30, 40 40)
{"type":"Point","coordinates":[125.6, 10.1]}
{"type":"Feature","properties":{"a": "b"},"geometry":{"type":"Point","coordinates":[125.6, 10.1]}}
{"type":"FeatureCollection","features":[{"type":"Feature","properties":{},"geometry":{"type":"Point","coordinates":[34.0,12.0]}},{"type":"Feature","properties":{},"geometry":{"type":"Point","coordinates":[78.0,56.0]}}]}
"#;

    let output = r#"{"coordinates":[34.0,12.0],"type":"Point"}
{"coordinates":[34.0,12.0],"type":"Point"}
{"coordinates":[[[-119.53125,33.75],[-118.125,33.75],[-118.125,35.15625],[-119.53125,35.15625],[-119.53125,33.75]]],"type":"Polygon"}
{"coordinates":[[30.0,10.0],[10.0,30.0],[40.0,40.0]],"type":"LineString"}
{"coordinates":[125.6,10.1],"type":"Point"}
{"coordinates":[125.6,10.1],"type":"Point"}
{"coordinates":[34.0,12.0],"type":"Point"}
{"coordinates":[78.0,56.0],"type":"Point"}
"#;
    Assert::main_binary()
        .with_args(&["gj", "geom"])
        .stdin(input)
        .stdout()
        .is(output)
        .unwrap();
}

#[test]
fn outputs_geojson_features() {
    let input = r#"12,34
12	34
9q5
LINESTRING (30 10, 10 30, 40 40)
{"type":"Point","coordinates":[125.6, 10.1]}
{"type":"Feature","properties":{"a": "b"},"geometry":{"type":"Point","coordinates":[125.6, 10.1]}}
{"type":"FeatureCollection","features":[{"type":"Feature","properties":{},"geometry":{"type":"Point","coordinates":[34.0,12.0]}},{"type":"Feature","properties":{},"geometry":{"type":"Point","coordinates":[78.0,56.0]}}]}
"#;

    let output = r#"{"geometry":{"coordinates":[34.0,12.0],"type":"Point"},"properties":{},"type":"Feature"}
{"geometry":{"coordinates":[34.0,12.0],"type":"Point"},"properties":{},"type":"Feature"}
{"geometry":{"coordinates":[[[-119.53125,33.75],[-118.125,33.75],[-118.125,35.15625],[-119.53125,35.15625],[-119.53125,33.75]]],"type":"Polygon"},"properties":{},"type":"Feature"}
{"geometry":{"coordinates":[[30.0,10.0],[10.0,30.0],[40.0,40.0]],"type":"LineString"},"properties":{},"type":"Feature"}
{"geometry":{"coordinates":[125.6,10.1],"type":"Point"},"properties":{},"type":"Feature"}
{"geometry":{"coordinates":[125.6,10.1],"type":"Point"},"properties":{"a":"b"},"type":"Feature"}
{"geometry":{"coordinates":[34.0,12.0],"type":"Point"},"properties":{},"type":"Feature"}
{"geometry":{"coordinates":[78.0,56.0],"type":"Point"},"properties":{},"type":"Feature"}
"#;
    Assert::main_binary()
        .with_args(&["gj", "f"])
        .stdin(input)
        .stdout()
        .is(output)
        .unwrap();
}

#[test]
fn outputs_geojson_featurecollection() {
    let input = r#"12,34
12	34
9q5
LINESTRING (30 10, 10 30, 40 40)
{"type":"Point","coordinates":[125.6, 10.1]}
{"type":"Feature","properties":{"a": "b"},"geometry":{"type":"Point","coordinates":[125.6, 10.1]}}
"#;

    let output = r#"{"features":[{"geometry":{"coordinates":[34.0,12.0],"type":"Point"},"properties":{},"type":"Feature"},{"geometry":{"coordinates":[34.0,12.0],"type":"Point"},"properties":{},"type":"Feature"},{"geometry":{"coordinates":[[[-119.53125,33.75],[-118.125,33.75],[-118.125,35.15625],[-119.53125,35.15625],[-119.53125,33.75]]],"type":"Polygon"},"properties":{},"type":"Feature"},{"geometry":{"coordinates":[[30.0,10.0],[10.0,30.0],[40.0,40.0]],"type":"LineString"},"properties":{},"type":"Feature"},{"geometry":{"coordinates":[125.6,10.1],"type":"Point"},"properties":{},"type":"Feature"},{"geometry":{"coordinates":[125.6,10.1],"type":"Point"},"properties":{"a":"b"},"type":"Feature"}],"type":"FeatureCollection"}"#;
    Assert::main_binary()
        .with_args(&["gj", "fc"])
        .stdin(input)
        .stdout()
        .is(output)
        .unwrap();
}

#[test]
fn outputs_geohash_for_point() {
    let input = r#"12,34
12	34
34,-118
"#;

    let output = r#"sf0hm8w
sf0hm8w
9qh16ve
"#;
    Assert::main_binary()
        .with_args(&["gh", "point", "7"])
        .stdin(input)
        .stdout()
        .is(output)
        .unwrap();
}

#[test]
fn geohash_not_allowed_for_non_point() {
    let input = "LINESTRING (30 10, 10 30, 40 40)\n";
    Assert::main_binary()
        .with_args(&["gh", "point", "7"])
        .stdin(input)
        .fails();
}

#[test]
fn geohash_children() {
    let input = "9q5\n";
    let output = r#"9q50
9q51
9q52
9q53
9q54
9q55
9q56
9q57
9q58
9q59
9q5b
9q5c
9q5d
9q5e
9q5f
9q5g
9q5h
9q5j
9q5k
9q5m
9q5n
9q5p
9q5q
9q5r
9q5s
9q5t
9q5u
9q5v
9q5w
9q5x
9q5y
9q5z
"#;
    Assert::main_binary()
        .with_args(&["gh", "children"])
        .stdin(input)
        .stdout()
        .is(output)
        .unwrap();

    Assert::main_binary()
        .with_args(&["gh", "children"])
        .stdin("LINESTRING (30 10, 10 30, 40 40)\n")
        .fails()
        .unwrap();
}
