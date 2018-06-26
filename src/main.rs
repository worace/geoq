#[macro_use]
extern crate lazy_static;
extern crate clap;
extern crate geo;
extern crate geohash;
extern crate regex;
extern crate wkt;

use clap::{App, ArgMatches, SubCommand};
use geo::{Coordinate, Geometry, LineString, Point, Polygon};
use regex::Regex;
use std::fmt;
use std::io;
use std::io::prelude::*;
use std::process;
// use wkt::ToWkt;

// Types to geom:
// lat/lon: split on comma -> parse to double -> geo::Point
// Geohash: strip whitespace -> geohash::decode()
// WKT: wkt::Wkt::from_str
// GeoJSON: geojson_str.parse::<GeoJson>()

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

    fn geom(&self) -> Geometry<f64> {
        match *self {
            Input::LatLon(ref raw) => {
                let pieces = raw.split(",").collect::<Vec<&str>>();
                let ll = match (pieces[0].parse::<f64>(), pieces[1].parse::<f64>()) {
                    (Ok(lat), Ok(lon)) => (lat, lon),
                    _ => (0.0, 0.0),
                };
                Geometry::Point(Point::new(ll.1, ll.0))
            }
            Input::Geohash(ref raw) => {
                let (bl, tr) = geohash::decode_bbox("ww8p1r4t8");
                println!("{:?}", bl);
                println!("{:?}", tr);
                let outer = LineString(vec![
                    Point::new(bl.x, bl.y),
                    Point::new(bl.x, tr.y),
                    Point::new(tr.x, tr.y),
                    Point::new(tr.x, bl.y),
                    Point::new(bl.x, bl.y),
                ]);
                Geometry::Polygon(Polygon::new(outer, Vec::new()))
            }
            _ => Geometry::Point(Point::new(0., 0.)),
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
        Geometry::Point(p) => {
            assert_eq!(p.0.y, 12.0);
            assert_eq!(p.0.x, 34.0);
        }
        _ => assert!(false),
    }
}

#[test]
fn geom_for_geohash() {
    let i = Input::Geohash("9q5".to_string());
    match i.geom() {
        Geometry::Polygon(p) => {
            println!("{:?}", p);
            // assert_eq!(p.0.y, 12.0);
            // assert_eq!(p.0.x, 34.0);
        }
        _ => assert!(false, "Geohash should give a polygon"),
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

fn run_wkt(_matches: &ArgMatches) -> Result<(), String> {
    println!("RUNNING WKT ***");
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let input = read_input(line.unwrap());
        let geom = input.geom();
        // let wkt = geom.to_wkt();
        println!("{:?}", geom);
        println!("{}", input.raw());
    }
    Ok(())
}

fn run_type(_matches: &ArgMatches) -> Result<(), String> {
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let input = read_input(line.unwrap());
        println!("{}", input);
    }
    Ok(())
}

fn run(matches: ArgMatches) -> Result<(), String> {
    match matches.subcommand() {
        ("wkt", Some(_m)) => run_wkt(&matches),
        ("type", Some(_m)) => run_type(&matches),
        _ => Err("Unknown Command".to_string()),
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
        println!("Application error: {}", e);
        process::exit(1);
    }
}
