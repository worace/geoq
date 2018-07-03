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
#[ignore]
fn it_outputs_wkt() {
    let input = "12,34";
    Assert::main_binary()
        .with_args(&["wkt"])
        .stdin(input)
        .stdout()
        .contains("POINT (34 12)")
        .unwrap();
}

#[test]
fn outputs_geojson_geoms() {
    let input = "12,34
12\t34
9q5
LINESTRING (30 10, 10 30, 40 40)
";

    let output = "{\"coordinates\":[34.0,12.0],\"type\":\"Point\"}
{\"coordinates\":[34.0,12.0],\"type\":\"Point\"}
{\"coordinates\":[[[-119.53125,33.75],[-118.125,33.75],[-118.125,35.15625],[-119.53125,35.15625],[-119.53125,33.75]]],\"type\":\"Polygon\"}
{\"coordinates\":[[30.0,10.0],[10.0,30.0],[40.0,40.0]],\"type\":\"LineString\"}
";
    Assert::main_binary()
        .with_args(&["gj", "geom"])
        .stdin(input)
        .stdout()
        .is(output)
        .unwrap();
}
