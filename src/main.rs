#[macro_use]
extern crate lazy_static;
extern crate clap;
extern crate geo;
extern crate geohash;
extern crate geojson;
extern crate regex;
extern crate wkt;

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

#[derive(Debug)]
enum Error {
    InvalidGeoJSON,
    NotImplemented,
    UnknownCommand,
    UnknownEntityFormat,
    InvalidWkt,
}

// Types to geom:
// [X] lat/lon: split on comma -> parse to double -> geo::Point
// [X] Geohash: strip whitespace -> geohash::decode()
// [*] WKT: wkt::Wkt::from_str -- BLOCKED on wkt library
// [ ] GeoJSON: geojson_str.parse::<GeoJson>()

#[derive(Debug)]
pub enum Input {
    LatLon(String),
    Geohash(String),
    WKT(String),
    GeoJSON(String),
    Unknown(String),
}

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
                let pieces = raw.split(",").collect::<Vec<&str>>();
                let ll = match (pieces[0].parse::<f64>(), pieces[1].parse::<f64>()) {
                    (Ok(lat), Ok(lon)) => (lat, lon),
                    // TODO Error
                    _ => (0.0, 0.0),
                };
                Ok(Geometry::Point(Point::new(ll.1, ll.0)))
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
                        let wkt_geom = wkt.items[0];
                        println!("{:?}", wkt.items.len());
                        Ok(Geometry::Point(Point::new(0.0, 0.0)))
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
            println!("{:?}", p);
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
    println!("*(**********)");
    println!("{:?}", i);
    match i.geom() {
        Ok(Geometry::LineString(l)) => {
            println!("{:?}", l);
            assert_eq!(l, expected);
        }
        Err(e) => assert!(false, "WKT error: {:?}", e),
        _ => assert!(false, "WKT should be read into a geometry"),
    }
}

lazy_static! {
    static ref LATLON: Regex = Regex::new(r"^-?\d+\.?\d*,-?\d+\.?\d*$").unwrap();
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

// fn get_wkt(geom: &Geometry<f64>) -> Option<wkt::Wkt> {
//     let mut wkt = wkt::Wkt::new();
//     // wkt.add_item(geom);
//     match geom {
//         Geometry::Point => Some(wkt::Geometry(geom)),
//         _ => None
//     }
// }

fn run_wkt(_matches: &ArgMatches) -> Result<(), Error> {
    println!("RUNNING WKT ***");
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let input = read_input(line.unwrap());
        let geom = input.geom();
        // let wkt = geom.to_wkt();
        match geom {
            Ok(g) => {
                println!("{:?}", g);
                println!("{}", input.raw());
            }
            Err(e) => return Err(e),
        }
    }
    Ok(())
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
        _ => Err(Error::UnknownCommand),
    }
}

fn main() {
    let version = "0.1";
    let matches = App::new("geoq")
        .version(version)
        .about("geoq - GeoSpatial utility belt")
        .subcommand(SubCommand::with_name("wkt").about("Output entity as WKT."))
        .subcommand(SubCommand::with_name("type").about("Check the format of an entity."))
        .get_matches();
    println!("{:?}", matches);
    println!("{:?}", matches.subcommand);

    if let Err(e) = run(matches) {
        println!("Application error: {:?}", e);
        process::exit(1);
    }
}
