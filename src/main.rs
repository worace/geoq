#[macro_use]
extern crate lazy_static;
extern crate clap;
extern crate regex;
use clap::{App, Arg, ArgMatches, SubCommand};
use regex::Regex;
use std::io;
use std::io::prelude::*;
use std::process;

#[derive(Debug)]
enum InputType {
    LatLon,
    Geohash,
    WKT,
    GeoJSON,
    Unknown,
}

struct Input {
    raw: String,
    input_type: InputType,
}

lazy_static! {
    static ref LATLON: Regex = Regex::new(r"^-?\d+\.?\d*,-?\d+\.?\d*$").unwrap();
    static ref GH: Regex = Regex::new(r"(?i)^[0-9a-z--a--i--l--o]+$").unwrap();
    static ref JSON: Regex = Regex::new(r"\{").unwrap();
    static ref WKT: Regex = Regex::new(r"(?ix)^point|linestring|polygon|multipoint|multilinestring|multipolygon").unwrap();
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

fn run_wkt(matches: &ArgMatches) -> Result<(), String> {
    println!("RUNNING WKT ***");
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let input = read_input(line.unwrap());
        println!("Input-contained line:");
        println!("{:?}", input.input_type);
        println!("{}", input.raw);
    }
    Ok(())
}

fn run_type(matches: &ArgMatches) -> Result<(), String> {
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let input = read_input(line.unwrap());
        println!("{:?}", input.input_type);
    }
    Ok(())
}

fn run(matches: ArgMatches) -> Result<(), String> {
    match matches.subcommand() {
        ("wkt", Some(m)) => run_wkt(&matches),
        ("type", Some(m)) => run_type(&matches),
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
