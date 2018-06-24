extern crate clap;
use clap::{App, Arg, ArgMatches, SubCommand};
use std::io;
use std::io::prelude::*;
use std::process;

enum InputType {
    LatLon,
    Geohash,
    WKT,
    GeoJSON,
}

struct Input {
    raw: String,
    input_type: InputType
}

fn read_input(line: String) -> Input {
    Input { raw: line, input_type: InputType::LatLon }
}

fn run_wkt(matches: &ArgMatches) -> Result<(), String> {
    println!("RUNNING WKT ***");
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let input = read_input(line.unwrap());
        println!("Input-contained line:");
        println!("{}", input.raw);
    }
    Ok(())
}

fn run(matches: ArgMatches) -> Result<(), String> {
    match matches.subcommand() {
        ("wkt", Some(m)) => run_wkt(&matches),
        _ => Err("Unknown Command".to_string()),
    }
}

fn main() {
    let version = "0.1";
    let matches = App::new("geoq")
        .version(version)
        .about("geoq - GeoSpatial utility belt")
        .subcommand(SubCommand::with_name("wkt").about("Output entity as WKT."))
        .get_matches();
    println!("{:?}", matches);
    println!("{:?}", matches.subcommand);
    println!("Hello, world!");

    if let Err(e) = run(matches) {
        println!("Application error: {}", e);
        process::exit(1);
    }
}
