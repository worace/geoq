extern crate assert_cli;
// extern crate geoq;
use assert_cli::Assert;

#[test]
fn it_gets_basic_debug_info_for_inputs() {
    let input = "12,34
12	34
9q5
{\"type\":\"Point\",\"coordinates\":[125.6, 10.1]}
LINESTRING (30 10, 10 30, 40 40)
";

    let output = r#"LatLon: 12,34
LatLon: 12	34
Geohash: 9q5
GeoJSON Geometry: {"type":"Point","coordinates":[125.6, 10.1]}
WKT: LINESTRING(30 10,10 30,40 40)
"#;
    Assert::main_binary()
        .with_args(&["read"])
        .stdin(input)
        .stdout()
        .contains(output)
        .unwrap();
}


#[test]
fn exits_on_invalid_input() {
    Assert::main_binary()
        .with_args(&["read"])
        .stdin("pizza")
        .stderr()
        .contains("UnknownEntityFormat")
        .fails()
        .unwrap();
}

#[test]
fn invalid_wkt() {
    let input = "Polygon ((30 10, 10 30, 40 40, 30 10)";
    Assert::main_binary()
        .with_args(&["read"])
        .stdin(input)
        .stderr()
        .is("Application error: InvalidWkt")
        .fails()
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
        .fails()
        .unwrap();
}

#[test]
fn geohash_fails_for_missing_or_invalid_level() {
    let input = "LINESTRING (30 10, 10 30, 40 40)\n";
    Assert::main_binary()
        .with_args(&["gh", "point", "pizza"])
        .stdin(input)
        .fails()
        .unwrap();

    Assert::main_binary()
        .with_args(&["gh", "point"])
        .stdin(input)
        .fails()
        .unwrap();
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

#[test]
fn geohash_neighbors() {
    let input = "9g3m\n";
    let output_with = r#"9g3m
9g3q
9g3w
9g3t
9g3s
9g3k
9g3h
9g3j
9g3n
"#;

    let output_without = r#"9g3q
9g3w
9g3t
9g3s
9g3k
9g3h
9g3j
9g3n
"#;
    Assert::main_binary()
        .with_args(&["gh", "neighbors"])
        .stdin(input)
        .stdout()
        .is(output_with)
        .unwrap();

    Assert::main_binary()
        .with_args(&["gh", "neighbors", "-e"])
        .stdin(input)
        .stdout()
        .is(output_without)
        .unwrap();

    Assert::main_binary()
        .with_args(&["gh", "neighbors"])
        .stdin("LINESTRING (30 10, 10 30, 40 40)\n")
        .fails()
        .unwrap();
}

#[test]
fn geohash_covering() {
    let input = r#"12,34
12	34
9q5
LINESTRING (30 10, 10 30, 40 40)
{"type":"Point","coordinates":[125.6, 10.1]}
{"type":"Feature","properties":{"a": "b"},"geometry":{"type":"Point","coordinates":[125.6, 10.1]}}
{"type":"FeatureCollection","features":[{"type":"Feature","properties":{},"geometry":{"type":"Point","coordinates":[34.0,12.0]}},{"type":"Feature","properties":{},"geometry":{"type":"Point","coordinates":[78.0,56.0]}}]}
"#;

    let output_with = r#"12,34
sf0
12	34
sf0
9q5
9qk
9qh
9q7
9q6
9q5
9q4
9mu
9mg
9mf
LINESTRING(30 10,10 30,40 40)
szh
sz5
syg
syf
syc
syb
sy8
swx
sww
swt
swm
swk
sw7
sw6
sw4
sw1
sw0
sqp
smz
smy
smv
smu
sms
sme
smd
sm9
sm3
sm2
sm0
skh
skd
skc
skb
sk9
sk7
sk6
sk5
sjr
sjp
se0
sdh
sdd
sdc
sdb
sd9
sd7
sd6
sd5
s9v
s9u
s7w
s7v
s7u
s7t
s7r
s7q
s7p
{"type":"Point","coordinates":[125.6, 10.1]}
wcc
{"type":"Feature","properties":{"a": "b"},"geometry":{"type":"Point","coordinates":[125.6, 10.1]}}
wcc
{"geometry":{"coordinates":[34.0,12.0],"type":"Point"},"properties":{},"type":"Feature"}
sf0
{"geometry":{"coordinates":[78.0,56.0],"type":"Point"},"properties":{},"type":"Feature"}
v9z
"#;

    let output_without = r#"sf0
sf0
9qk
9qh
9q7
9q6
9q5
9q4
9mu
9mg
9mf
szh
sz5
syg
syf
syc
syb
sy8
swx
sww
swt
swm
swk
sw7
sw6
sw4
sw1
sw0
sqp
smz
smy
smv
smu
sms
sme
smd
sm9
sm3
sm2
sm0
skh
skd
skc
skb
sk9
sk7
sk6
sk5
sjr
sjp
se0
sdh
sdd
sdc
sdb
sd9
sd7
sd6
sd5
s9v
s9u
s7w
s7v
s7u
s7t
s7r
s7q
s7p
wcc
wcc
sf0
v9z
"#;
    Assert::main_binary()
        .with_args(&["gh", "covering", "3"])
        .stdin(input)
        .stdout()
        .is(output_without)
        .unwrap();

    Assert::main_binary()
        .with_args(&["gh", "covering", "3", "-o"])
        .stdin(input)
        .stdout()
        .is(output_with)
        .unwrap();
}

#[test]
fn gh_covering_errors() {
    let input = "LINESTRING (30 10, 10 30, 40 40)\n";
    Assert::main_binary()
        .with_args(&["gh", "covering", "pizza"])
        .stdin(input)
        .fails()
        .unwrap();

    Assert::main_binary()
        .with_args(&["gh", "covering"])
        .stdin(input)
        .fails()
        .unwrap();
}

#[test]
fn gj_geom_edge_case() {
    let input = r#"{"coordinates":[[[-87.97874531338036,34.92420945798039],[-87.9785957342853,34.92418491767909],[-87.97857664070615,34.9242631544226],[-87.97872621980122,34.92428769470051],[-87.97874531338036,34.92420945798039]]],"type":"Polygon"}
"#;
    Assert::main_binary()
        .with_args(&["gj", "geom"])
        .stdin(input)
        .stdout()
        .is(input)
        .unwrap();

}

#[test]
fn filter_intersects() {
    let input = r#"34.2277,-118.2623
{"type":"Polygon","coordinates":[[[-117.87231445312499,34.77997173591062],[-117.69653320312499,34.77997173591062],[-117.69653320312499,34.90170042871546],[-117.87231445312499,34.90170042871546],[-117.87231445312499,34.77997173591062]]]}
{"type":"Polygon","coordinates":[[[-118.27880859375001,34.522398580663314],[-117.89154052734375,34.522398580663314],[-117.89154052734375,34.649025753526985],[-118.27880859375001,34.649025753526985],[-118.27880859375001,34.522398580663314]]]}
"#;

    let output = r#"34.2277,-118.2623
{"type":"Polygon","coordinates":[[[-118.27880859375001,34.522398580663314],[-117.89154052734375,34.522398580663314],[-117.89154052734375,34.649025753526985],[-118.27880859375001,34.649025753526985],[-118.27880859375001,34.522398580663314]]]}
"#;

    Assert::main_binary()
        .with_args(&["filter", "intersects", "9q5"])
        .stdin(input)
        .stdout()
        .is(output)
        .unwrap();

}

#[test]
#[ignore]
fn reading_geojson_feature_without_properties() {
    let input = r#"{"type":"Feature","geometry":{"type":"Point","coordinates":[125.6, 10.1]}}
{"type":"Feature","properties":null,"geometry":{"type":"Point","coordinates":[125.6, 10.1]}}
"#;

    Assert::main_binary()
        .with_args(&["wkt"])
        .stdin(input)
        .stderr()
        .is("")
        .unwrap();
}


#[test]
fn json_point() {
    let input = r#"{"latitude": 34.3, "longitude": -118.2, "name": "Horace", "pizza": "pie"}
{"lat": 34.3, "lon": -118.2, "name": "Horace", "pizza": "pie"}
{"latitude": 34.3, "lng": -118.2, "name": "Horace", "pizza": "pie"}
"#;

    let output = r#"{"geometry":{"coordinates":[-118.2,34.3],"type":"Point"},"properties":{"latitude":34.3,"longitude":-118.2,"name":"Horace","pizza":"pie"},"type":"Feature"}
{"geometry":{"coordinates":[-118.2,34.3],"type":"Point"},"properties":{"lat":34.3,"lon":-118.2,"name":"Horace","pizza":"pie"},"type":"Feature"}
{"geometry":{"coordinates":[-118.2,34.3],"type":"Point"},"properties":{"latitude":34.3,"lng":-118.2,"name":"Horace","pizza":"pie"},"type":"Feature"}
"#;

    Assert::main_binary().with_args(&["json", "point"]).stdin(input)
        .stdout().is(output).unwrap();

    Assert::main_binary().with_args(&["json", "point"])
        .stdin("pizza").fails().unwrap();

    Assert::main_binary().with_args(&["json", "point"])
        .stdin("[\"not-json-object\"]").fails().unwrap();
    Assert::main_binary().with_args(&["json", "point"])
        .stdin("{\"no-lat-lon\": \"hi\"}").fails().unwrap();
}
