#[macro_use]
extern crate lazy_static;
extern crate clap;
extern crate geo;
extern crate geo_types;
extern crate geohash;
extern crate geojson;
extern crate geoq_wkt;
extern crate os_type;
extern crate percent_encoding;
extern crate regex;
extern crate serde_json;

mod geoq;
use geoq::commands;
use geoq::error::Error;

use clap::{App, Arg, ArgMatches, SubCommand};
use std::process;

fn run(matches: ArgMatches) -> Result<(), Error> {
    match matches.subcommand() {
        ("wkt", Some(_)) => commands::wkt::run(),
        ("debug", Some(_m)) => commands::debug::run(),
        ("gj", Some(m)) => commands::geojson::run(m),
        ("gh", Some(m)) => commands::geohash::run(m),
        ("map", Some(_)) => commands::map::run(),
        ("filter", Some(m)) => commands::filter::run(m),
        _ => Err(Error::UnknownCommand),
    }
}

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

fn main() {
    let geojson = SubCommand::with_name("gj")
        .about("Output entity as GeoJSON")
        .subcommand(SubCommand::with_name("geom").about("Output entity as a GeoJSON geometry"))
        .subcommand(SubCommand::with_name("f").about("Output entity as a GeoJSON Feature"))
        .subcommand(
            SubCommand::with_name("fc")
                .about("Collect all given entities into a GeoJSON Feature Collection"),
        );

    let geohash = SubCommand::with_name("gh")
        .about("Output Geohash representations of entities")
        .subcommand(
            SubCommand::with_name("point")
                .about("Output base 32 Geohash for a given Lat,Lon")
                .arg(
                    Arg::with_name("level")
                        .help("Characters of geohash precision")
                        .required(true)
                        .index(1),
                ),
        )
        .subcommand(
            SubCommand::with_name("covering")
                .about("Output the set of geohashes at the given level which covers the given entity")
                .arg(
                    Arg::with_name("level")
                        .help("Characters of geohash precision")
                        .required(true)
                        .index(1),
                ).arg(Arg::with_name("original")
                      .long("original")
                      .short("o")
                      .help("Also print the query entity in the output.\nUseful for mapping a geometry along with its covering Geohashes.")),
        )
        .subcommand(SubCommand::with_name("children").about("Get children for the given geohash"))
        .subcommand(SubCommand::with_name("neighbors")
                    .about("Get neighbors of the given Geohash")
                    .arg(Arg::with_name("exclude")
                         .long("exclude")
                         .short("e")
                         .help("Exclude the given geohash from its neighbors.\nBy default it will be included in the output,\ngiving a 3x3 grid centered on the provided geohash.")));

    let filter = SubCommand::with_name("filter")
        .about("Select a subset of provided entities based on geospatial predicates")
        .subcommand(
            SubCommand::with_name("intersects")
                .about("Output only entities (from STDIN) which intersect a QUERY entity (as command-line ARG)")
                .arg(
                    Arg::with_name("query")
                        .help("Entity to check intersections.\nMust be Lat/Lon, Geohash, WKT, or GeoJSON.\nCurrently only POLYGON query geometries are supported.")
                        .required(true)
                        .index(1),
                )
        );

    let matches = App::new("geoq")
        .version(VERSION)
        .about("geoq - GeoSpatial utility belt")
        .subcommand(SubCommand::with_name("wkt").about("Output entity as WKT"))
        .subcommand(SubCommand::with_name("debug").about("Check the format of an entity"))
        .subcommand(SubCommand::with_name("map").about("View entities on a map using geojson.io"))
        .subcommand(geohash)
        .subcommand(geojson)
        .subcommand(filter)
        .get_matches();

    if let Err(e) = run(matches) {
        eprintln!("Application error: {:?}", e);
        process::exit(1);
    }
}
