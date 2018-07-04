#[macro_use]
extern crate lazy_static;
extern crate clap;
extern crate geo;
extern crate geo_types;
extern crate geojson;
extern crate regex;
extern crate serde_json;
extern crate wkt;

mod geoq;
use geoq::error::Error;
use geoq::reader::Reader;
use geoq::entity;

use clap::{App, ArgMatches, SubCommand};
use geojson::GeoJson;
use std::io;
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
    let mut stdin_reader = stdin.lock();
    let reader = Reader::new(&mut stdin_reader);
    let entities = reader.flat_map(|i| entity::from_input(i));
    for e in entities {
        let g = e.geom();
        eprintln!("{:?}", g);
    }
    Ok(())
}

fn run_geojson_geom(_matches: &ArgMatches) -> Result<(), Error> {
    let stdin = io::stdin();
    let mut stdin_reader = stdin.lock();
    let reader = Reader::new(&mut stdin_reader);
    let entities = reader.flat_map(|i| entity::from_input(i));
    for e in entities {
        let gj_geom = e.geojson_geometry();
        println!("{}", serde_json::to_string(&gj_geom).unwrap());
    }
    Ok(())
}

fn run_geojson_feature(_matches: &ArgMatches) -> Result<(), Error> {
    let stdin = io::stdin();
    for input in Reader::new(&mut stdin.lock()) {
        let geom = input.geom();
        match geom {
            Ok(g) => {
                let gj_geom = geojson::Geometry::new(geojson::Value::from(&g));
                let feature = GeoJson::Feature(geojson::Feature {
                    bbox: None,
                    geometry: Some(gj_geom),
                    id: None,
                    properties: None,
                    foreign_members: None,
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
        _ => Err(Error::UnknownCommand),
    }
}

fn run_type(_matches: &ArgMatches) -> Result<(), Error> {
    let stdin = io::stdin();
    for input in Reader::new(&mut stdin.lock()) {
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
