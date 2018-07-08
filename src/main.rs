#[macro_use]
extern crate lazy_static;
extern crate clap;
extern crate geo;
extern crate geo_types;
extern crate geohash;
extern crate geojson;
extern crate regex;
extern crate serde_json;
extern crate url;
extern crate wkt;
extern crate os_type;

mod geoq;
use geoq::entity;
use geoq::error::Error;
use geoq::reader::Reader;
use geoq::input::Input;
use geoq::input;
use std::process::Command;
use url::percent_encoding;
use url::percent_encoding::{utf8_percent_encode, DEFAULT_ENCODE_SET};
use geo_types::{Geometry, Polygon, MultiPolygon};

use clap::{App, Arg, ArgMatches, SubCommand};
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
        println!("{}", e.wkt());
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
    let mut stdin_reader = stdin.lock();
    let reader = Reader::new(&mut stdin_reader);
    let entities = reader.flat_map(|i| entity::from_input(i));
    for e in entities {
        let f = e.geojson_feature();
        println!("{}", serde_json::to_string(&f).unwrap());
    }
    Ok(())
}

fn run_geojson_feature_collection(_matches: &ArgMatches) -> Result<(), Error> {
    let stdin = io::stdin();
    let mut stdin_reader = stdin.lock();
    let reader = Reader::new(&mut stdin_reader);
    let features = reader
        .flat_map(|i| entity::from_input(i))
        .map(|e| e.geojson_feature());

    let fc = geojson::FeatureCollection {
        bbox: None,
        features: features.collect(),
        foreign_members: None,
    };
    println!("{}", GeoJson::from(fc).to_string());

    Ok(())
}

fn run_geojson(matches: &ArgMatches) -> Result<(), Error> {
    let (_, gj) = matches.subcommand();
    match gj.unwrap().subcommand() {
        ("geom", Some(_m)) => run_geojson_geom(&matches),
        ("f", Some(_m)) => run_geojson_feature(&matches),
        ("fc", Some(_m)) => run_geojson_feature_collection(&matches),
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

fn run_geohash_point(matches: &ArgMatches) -> Result<(), Error> {
    match matches.value_of("level") {
        Some(l) => match l.parse::<usize>() {
            Ok(level) => {
                let stdin = io::stdin();
                let mut stdin_reader = stdin.lock();
                let reader = Reader::new(&mut stdin_reader);
                let entities = reader.flat_map(|i| entity::from_input(i));
                for e in entities {
                    match e.geom() {
                        geo_types::Geometry::Point(p) => {
                            println!("{}", geohash::encode(p.0, level));
                        }
                        _ => return Err(Error::NotImplemented),
                    }
                }
                Ok(())
            }
            _ => Err(Error::InvalidNumberFormat),
        },
        _ => Err(Error::MissingArgument),
    }
}

fn run_geohash_covering(matches: &ArgMatches) -> Result<(), Error> {
    match matches.value_of("level") {
        Some(l) => match l.parse::<usize>() {
            Ok(level) => {
                let stdin = io::stdin();
                let mut stdin_reader = stdin.lock();
                let reader = Reader::new(&mut stdin_reader);
                for i in reader {
                    if matches.is_present("original") {
                        println!("{}", i.raw());
                    }
                    for entity in entity::from_input(i) {
                        let g = entity.geom();
                        for gh in geoq::geohash::covering(&g, level) {
                            println!("{}", gh);
                        }
                    }
                }
                Ok(())
            }
            _ => Err(Error::InvalidNumberFormat),
        },
        _ => Err(Error::MissingArgument),
    }
}

fn run_geohash_children() -> Result<(), Error> {
    let stdin = io::stdin();
    let mut stdin_reader = stdin.lock();
    let reader = Reader::new(&mut stdin_reader);
    for i in reader {
        match i {
            Input::Geohash(ref raw) => {
                for gh in geoq::geohash::children(raw) {
                    println!("{}", gh);
                }
            }
            _ => return Err(Error::NotImplemented),
        }
    }
    Ok(())
}

fn run_geohash_neighbors(matches: &ArgMatches) -> Result<(), Error> {
    let exclude = matches.is_present("exclude");
    let stdin = io::stdin();
    let mut stdin_reader = stdin.lock();
    let reader = Reader::new(&mut stdin_reader);
    for i in reader {
        match i {
            Input::Geohash(ref raw) => {
                for gh in geoq::geohash::neighbors(raw, !exclude).iter() {
                    println!("{}", gh);
                }
            }
            _ => return Err(Error::NotImplemented),
        }
    }
    Ok(())
}

fn run_geohash(matches: &ArgMatches) -> Result<(), Error> {
    match matches.subcommand() {
        ("point", Some(m)) => run_geohash_point(m),
        ("children", Some(_)) => run_geohash_children(),
        ("neighbors", Some(m)) => run_geohash_neighbors(m),
        ("covering", Some(m)) => run_geohash_covering(m),
        _ => Err(Error::UnknownCommand),
    }
}

fn run_map() -> Result<(), Error> {
    let stdin = io::stdin();
    let mut stdin_reader = stdin.lock();
    let reader = Reader::new(&mut stdin_reader);
    let features = reader
        .flat_map(|i| entity::from_input(i))
        .map(|e| e.geojson_feature());

    let fc = geojson::FeatureCollection {
        bbox: None,
        features: features.collect(),
        foreign_members: None,
    };
    let fc_json = GeoJson::from(fc).to_string();
    let encoded = utf8_percent_encode(&fc_json, DEFAULT_ENCODE_SET);
    let url = format!("http://geojson.io#data=data:application/json,{}", encoded);

    // TODO: something for windows here?
    let open_command = match os_type::current_platform().os_type {
        os_type::OSType::OSX => "open",
        _ => "xdg-open"
    };

    Command::new(open_command)
        .arg(url)
        .status()
        .expect("Failed to open geojson.io");

    Ok(())
}

fn run_filter_intersects(matches: &ArgMatches) -> Result<(), Error> {
    match matches.value_of("query") {
        Some(q) => {
            let query_entities = entity::from_input(input::read_line(q.to_string()));
            if query_entities.is_empty() {
                Err(Error::UnknownEntityFormat)
            } else {
                let query_geoms = query_entities.into_iter().map(|e| e.geom());
                let query_polygons: Vec<Polygon<f64>> = query_geoms.flat_map(|g| {
                    match g {
                        Geometry::Polygon(p) => vec![p],
                        Geometry::MultiPolygon(mp) => mp.0,
                        _ => vec![]
                    }
                }).collect();

                let stdin = io::stdin();
                let mut stdin_reader = stdin.lock();
                let reader = Reader::new(&mut stdin_reader);

                for input in reader {
                    // TODO restructure so this doesnlt need to be cloned
                    let output = input.raw().clone();
                    let entities = entity::from_input(input);
                    let geoms: Vec<Geometry<f64>> = entities.into_iter().map(|e| e.geom()).collect();
                    if query_polygons.iter().any(|ref query_poly| {
                        geoms.iter().any(|ref e_geom| geoq::geohash::intersects(query_poly, e_geom))
                    }) {
                        println!("{}", output);
                    }
                }

                Ok(())
            }
        },
        _ => Err(Error::MissingArgument),
    }
}

fn run_filter(matches: &ArgMatches) -> Result<(), Error> {
    match matches.subcommand() {
        ("intersects", Some(m)) => run_filter_intersects(m),
        _ => Err(Error::UnknownCommand),
    }
}

fn run(matches: ArgMatches) -> Result<(), Error> {
    match matches.subcommand() {
        ("wkt", Some(_m)) => run_wkt(&matches),
        ("type", Some(_m)) => run_type(&matches),
        ("gj", Some(_m)) => run_geojson(&matches),
        ("gh", Some(m)) => run_geohash(m),
        ("map", Some(_)) => run_map(),
        ("filter", Some(m)) => run_filter(m),
        _ => Err(Error::UnknownCommand),
    }
}

fn main() {
    let version = "0.1";

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
        .version(version)
        .about("geoq - GeoSpatial utility belt")
        .subcommand(SubCommand::with_name("wkt").about("Output entity as WKT"))
        .subcommand(SubCommand::with_name("type").about("Check the format of an entity"))
        .subcommand(SubCommand::with_name("map").about("View entities on a map using geojson.io"))
        .subcommand(geohash)
        .subcommand(geojson)
        .subcommand(filter)
        .get_matches();

    if let Err(e) = run(matches) {
        println!("Application error: {:?}", e);
        process::exit(1);
    }
}
