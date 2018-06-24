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
    Assert::main_binary()
        .with_args(&["type"])
        .stdin(input)
        .stdout()
        .contains("LatLon\nGeohash\nGeoJSON\nWKT\nUnknown")
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
