#[macro_use]
extern crate lazy_static;
extern crate clap;
extern crate geo;
extern crate geohash;
extern crate geo_types;
extern crate geojson;
extern crate regex;
extern crate wkt;
extern crate serde_json;

use clap::{App, ArgMatches, SubCommand};
use geo::{Geometry, LineString, Point, Polygon};
use geojson::conversion::*;
use geojson::GeoJson;
use regex::Regex;
use std::fmt;
use std::io;
use std::io::prelude::*;
use std::process;
use wkt::Wkt;
use wkt::ToGeo;

#[derive(Debug)]
enum Error {
    InvalidGeoJSON,
    NotImplemented,
    UnknownCommand,
    UnknownEntityFormat,
    InvalidWkt,
}

#[derive(Debug)]
pub enum Input {
    LatLon(String),
    Geohash(String),
    WKT(String),
    GeoJSON(String),
    Unknown(String),
}

// Decoding
// - Input: 1 per line; contains Raw as string
// - Entity: multiple per Input.
//   Probably(?) contains each native de-serialized type as member
//   (so Entity::GeoJson(GeoJson::Geometry/Feature, Entity::WKT(Wkt::Thin, Entity::WKT(Wkt::Thing)))
// - io Line --Map--> Input --FlatMap--> Entity
// Entity traits:
// - to geo_types Geom
// - to wkt
// - to geojson feature / geometry
// Building Feature Collection:
// Iterate entities from stream
// Start serde json with type: Feature Collection, features: (start list)
// for each entity build feature and write to serde list
// Interface Migration
// - InputReader(std IO) -> Iterator<Input>
//   - replace all repeated instances of iterating over stdin lines with this
// - InputReader.entities -> Iterator<Entity>
// Remove Input::Uknown -- just make read_input give Result and surface these errors earlier

impl Input {
    fn raw(&self) -> &String {
        match *self {
            Input::LatLon(ref raw) => raw,
            Input::Geohash(ref raw) => raw,
            Input::WKT(ref raw) => raw,
            Input::GeoJSON(ref raw) => raw,
            Input::Unknown(ref raw) => raw,
        }
    }

    fn geom(&self) -> Result<Geometry<f64>, Error> {
        match *self {
            Input::LatLon(ref raw) => {
                let pieces = LATLON_SPLIT.split(raw).collect::<Vec<&str>>();
                match (pieces[0].parse::<f64>(), pieces[1].parse::<f64>()) {
                    (Ok(lat), Ok(lon)) => Ok(Geometry::Point(Point::new(lon, lat))),
                    _ => Err(Error::NotImplemented),
                }
            }
            Input::Geohash(ref raw) => {
                let (bl, tr) = geohash::decode_bbox(raw);
                let outer = LineString(vec![
                    Point::new(bl.x, bl.y),
                    Point::new(tr.x, bl.y),
                    Point::new(tr.x, tr.y),
                    Point::new(bl.x, tr.y),
                    Point::new(bl.x, bl.y),
                ]);
                Ok(Geometry::Polygon(Polygon::new(outer, Vec::new())))
            }
            Input::GeoJSON(ref raw) => {
                let gj = match raw.parse() {
                    Ok(gj) => gj,
                    Err(_) => {
                        return Err(Error::InvalidGeoJSON);
                    }
                };

                match gj {
                    GeoJson::Geometry(gj_geom) => {
                        let geom: Result<Geometry<f64>, geojson::Error> = gj_geom.value.try_into();
                        match geom {
                            Ok(g) => Ok(g),
                            Err(_) => Err(Error::InvalidGeoJSON),
                        }
                    }
                    GeoJson::Feature(_feature) => Err(Error::NotImplemented),
                    GeoJson::FeatureCollection(_fc) => Err(Error::NotImplemented),
                }
            }
            Input::WKT(ref raw) => {
                let wkt_res: Result<Wkt<f64>, &str> = Wkt::from_str(raw);
                match wkt_res {
                    Ok(wkt) => {
                        match wkt.items.get(0) {
                            Some(wkt_geom) => {
                                let geo_geom = wkt_geom.to_geo();
                                match geo_geom {
                                    Ok(res) => Ok(res),
                                    Err(_) => Err(Error::InvalidWkt)
                                }
                            }
                            _ => Err(Error::InvalidWkt)
                        }
                    }
                    Err(_) => Err(Error::InvalidWkt),
                }
            }
            _ => Err(Error::UnknownEntityFormat),
        }
    }
}

impl fmt::Display for Input {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Input::LatLon(ref raw) => write!(f, "LatLon({})", raw),
            Input::Geohash(ref raw) => write!(f, "Geohash({})", raw),
            Input::WKT(ref raw) => write!(f, "WKT({})", raw),
            Input::GeoJSON(ref raw) => write!(f, "GeoJSON({})", raw),
            Input::Unknown(ref raw) => write!(f, "Unknown({})", raw),
        }
    }
}

#[test]
fn reading_input_formats() {
    match read_input("12,34".to_string()) {
        Input::LatLon(_) => assert!(true),
        _ => assert!(false)
    }
    match read_input("12\t34".to_string()) {
        Input::LatLon(_) => assert!(true),
        _ => assert!(false)
    }
}

#[test]
fn geom_for_lat_lon() {
    let i = Input::LatLon("12,34".to_string());
    match i.geom() {
        Ok(Geometry::Point(p)) => {
            assert_eq!(p.0.y, 12.0);
            assert_eq!(p.0.x, 34.0);
        }
        _ => assert!(false),
    }
}

#[test]
fn geom_for_lat_lon_tsv() {
    let i = Input::LatLon("12\t34".to_string());
    match i.geom() {
        Ok(Geometry::Point(p)) => {
            assert_eq!(p.0.y, 12.0);
            assert_eq!(p.0.x, 34.0);
        }
        _ => assert!(false),
    }
}

#[test]
fn geom_for_geohash() {
    let expected = Polygon::new(
        vec![
            [-119.53125, 33.75],
            [-118.125, 33.75],
            [-118.125, 35.15625],
            [-119.53125, 35.15625],
            [-119.53125, 33.75],
        ].into(),
        vec![],
    );

    let i = Input::Geohash("9q5".to_string());
    match i.geom() {
        Ok(Geometry::Polygon(p)) => {
            assert_eq!(p, expected);
        }
        _ => assert!(false, "Geohash should give a polygon"),
    }
}

#[test]
fn geom_for_geojson() {
    let gj = "{\"type\": \"LineString\", \"coordinates\": [[-26.01, 59.17], [-15.46, 45.58], [0.35, 35.74]]}";
    let expected = LineString(
        vec![
            Point::new(-26.01, 59.17),
            Point::new(-15.46, 45.58),
            Point::new(0.35, 35.74),
        ].into(),
    );
    let i = Input::GeoJSON(gj.to_string());
    match i.geom() {
        Ok(Geometry::LineString(l)) => {
            assert_eq!(l, expected);
        }
        _ => assert!(false, "Geohash should give a polygon"),
    }
}

#[test]
fn geom_for_invalid_geojson() {
    let gj = "{pizza}";
    let i = Input::GeoJSON(gj.to_string());
    match i.geom() {
        Err(Error::InvalidGeoJSON) => assert!(true, "Returns proper error"),
        _ => assert!(
            false,
            "Reading invalid GeoJSON should give Error::InvalidGeoJSON"
        ),
    }
}

#[test]
fn geom_for_wkt() {
    let expected = LineString(
        vec![
            Point::new(30.0, 10.0),
            Point::new(10.0, 30.0),
            Point::new(40.0, 40.0),
        ].into(),
    );

    let wkt = "LINESTRING (30 10, 10 30, 40 40)";
    let i = Input::WKT(String::from(wkt));
    match i.geom() {
        Ok(Geometry::LineString(l)) => {
            assert_eq!(l, expected);
        }
        Err(e) => assert!(false, "WKT error: {:?}", e),
        _ => assert!(false, "WKT should be read into a geometry"),
    }
}

lazy_static! {
    static ref LATLON_SPLIT: Regex = Regex::new(",|\t").unwrap();
    static ref LATLON: Regex = Regex::new(r"^-?\d+\.?\d*[,\t]-?\d+\.?\d*$").unwrap();
    static ref GH: Regex = Regex::new(r"(?i)^[0-9a-z--a--i--l--o]+$").unwrap();
    static ref JSON: Regex = Regex::new(r"\{").unwrap();
    static ref WKT: Regex = Regex::new(
        r"(?ix)^point|linestring|polygon|multipoint|multilinestring|multipolygon"
    ).unwrap();
}

fn read_input(line: String) -> Input {
    if LATLON.is_match(&line) {
        Input::LatLon(line)
    } else if GH.is_match(&line) {
        Input::Geohash(line)
    } else if JSON.is_match(&line) {
        Input::GeoJSON(line)
    } else if WKT.is_match(&line) {
        Input::WKT(line)
    } else {
        Input::Unknown(line)
    }
}

fn run_wkt(_matches: &ArgMatches) -> Result<(), Error> {
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let input = read_input(line.unwrap());
        let geom = input.geom();
        // let wkt = geom.to_wkt();
        match geom {
            Ok(g) => {
                eprintln!("{:?}", g);
                eprintln!("{}", input.raw());
            }
            Err(e) => return Err(e),
        }
    }
    Ok(())
}

fn run_geojson_geom(_matches: &ArgMatches) -> Result<(), Error> {
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let input = read_input(line.unwrap());
        let geom = input.geom();
        match geom {
            Ok(g) => {
                let gj_geom = geojson::Geometry::new(geojson::Value::from(&g));
                println!("{}", serde_json::to_string(&gj_geom).unwrap());
                // match gj {
                //     serde_json::Value => println!("{}", gj.to_string()),
                //     _ => return Err(Error::InvalidGeoJSON)
                // }
            }
            Err(e) => return Err(e),
        }
    }
    Ok(())
}

fn run_geojson_feature(_matches: &ArgMatches) -> Result<(), Error> {
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let input = read_input(line.unwrap());
        let geom = input.geom();
        match geom {
            Ok(g) => {
                let gj_geom = geojson::Geometry::new(geojson::Value::from(&g));
                let feature = GeoJson::Feature(geojson::Feature {
                    bbox: None,
                    geometry: Some(gj_geom),
                    id: None,
                    properties: None,
                    foreign_members: None
                });
                println!("{}", serde_json::to_string(&feature).unwrap());
            }
            Err(e) => return Err(e),
        }
    }
    Ok(())
}

fn run_geojson(matches: &ArgMatches) -> Result<(), Error> {
    let (_, gj) = matches.subcommand();
    match gj.unwrap().subcommand() {
        ("geom", Some(_m)) => run_geojson_geom(&matches),
        ("f", Some(_m)) => run_geojson_feature(&matches),
        _ => Err(Error::UnknownCommand)
    }
}

fn run_type(_matches: &ArgMatches) -> Result<(), Error> {
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let input = read_input(line.unwrap());
        println!("{}", input);
    }
    Ok(())
}

fn run(matches: ArgMatches) -> Result<(), Error> {
    match matches.subcommand() {
        ("wkt", Some(_m)) => run_wkt(&matches),
        ("type", Some(_m)) => run_type(&matches),
        ("gj", Some(_m)) => run_geojson(&matches),
        _ => Err(Error::UnknownCommand),
    }
}

fn main() {
    let version = "0.1";

    let geojson = SubCommand::with_name("gj")
        .about("Output entity as GeoJSON.")
        .subcommand(SubCommand::with_name("geom").about("Output entity as a GeoJSON geometry."))
        .subcommand(SubCommand::with_name("f").about("Output entity as a GeoJSON Feature."));
    let matches = App::new("geoq")
        .version(version)
        .about("geoq - GeoSpatial utility belt")
        .subcommand(SubCommand::with_name("wkt").about("Output entity as WKT."))
        .subcommand(SubCommand::with_name("type").about("Check the format of an entity."))
        .subcommand(geojson)
        .get_matches();

    if let Err(e) = run(matches) {
        println!("Application error: {:?}", e);
        process::exit(1);
    }
}
