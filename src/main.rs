#[macro_use]
extern crate lazy_static;
extern crate clap;
extern crate geo;
extern crate geo_types;
extern crate geojson;
extern crate regex;
extern crate wkt;
extern crate serde_json;

mod input;
mod error;
use input::Input;
use error::Error;

use clap::{App, ArgMatches, SubCommand};
use geojson::GeoJson;
use std::io;
use std::io::prelude::*;
use std::process;

// Decoding
// - Input: 1 per line; contains Raw as string
// - Entity: multiple per Input.
//   Probably(?) contains each native de-serialized type as member
//   (so Entity::GeoJson(GeoJson::Geometry/Feature, Entity::WKT(Wkt::Thing, Entity::WKT(Wkt::Thing)))
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

fn run_wkt(_matches: &ArgMatches) -> Result<(), Error> {
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let input = input::read_line(line.unwrap());
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
        let input = input::read_line(line.unwrap());
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
        let input = input::read_line(line.unwrap());
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
        let input = input::read_line(line.unwrap());
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
