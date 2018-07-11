extern crate serde_json;
extern crate geojson;
extern crate clap;

use clap::ArgMatches;
use geojson::GeoJson;
use geoq::error::Error;
use geoq::reader::Reader;
use geoq::entity;
use std::io;

fn geom() -> Result<(), Error> {
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

fn feature() -> Result<(), Error> {
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

fn feature_collection() -> Result<(), Error> {
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

pub fn run_geojson(matches: &ArgMatches) -> Result<(), Error> {
    let (_, gj) = matches.subcommand();
    match gj.unwrap().subcommand() {
        ("geom", Some(_)) => geom(),
        ("f", Some(_)) => feature(),
        ("fc", Some(_)) => feature_collection(),
        _ => Err(Error::UnknownCommand),
    }
}
