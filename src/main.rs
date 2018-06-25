#[macro_use]
extern crate lazy_static;
extern crate clap;
extern crate geo;
extern crate regex;
extern crate wkt;

use clap::{App, ArgMatches, SubCommand};
use geo::{Geometry, Point};
use regex::Regex;
use std::io;
use std::io::prelude::*;
use std::process;
use wkt::ToWkt;

// Types to geom:
// lat/lon: split on comma -> parse to double -> geo::Point
// Geohash: strip whitespace -> geohash::decode()
// WKT: wkt::Wkt::from_str
// GeoJSON: geojson_str.parse::<GeoJson>()

#[derive(Debug)]
enum InputType {
    LatLon,
    Geohash,
    WKT,
    GeoJSON,
    Unknown,
}

pub struct Input {
    raw: String,
    input_type: InputType,
}

pub fn get_geom(input: &Input) -> Geometry<f64> {
    Geometry::Point(Point::new(0., 0.))
}

#[test]
fn getting_geometries() {
    println!("*******************");
    println!("*******************");
    println!("*******************");
    let i = Input {
        raw: "12,34".to_string(),
        input_type: InputType::LatLon,
    };
    let g: Geometry<f64> = get_geom(&i);
    println!("{:?}", g);
    assert!(true);
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
        Input {
            raw: line,
            input_type: InputType::LatLon,
        }
    } else if GH.is_match(&line) {
        Input {
            raw: line,
            input_type: InputType::Geohash,
        }
    } else if JSON.is_match(&line) {
        Input {
            raw: line,
            input_type: InputType::GeoJSON,
        }
    } else if WKT.is_match(&line) {
        Input {
            raw: line,
            input_type: InputType::WKT,
        }
    } else {
        Input {
            raw: line,
            input_type: InputType::Unknown,
        }
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
        let geom = get_geom(&input);
        let wkt = geom.to_wkt();
        println!("{:?}", geom);
        println!("{}", input.raw);
    }
    Ok(())
}

fn run_type(_matches: &ArgMatches) -> Result<(), String> {
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let input = read_input(line.unwrap());
        println!("{:?}", input.input_type);
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
