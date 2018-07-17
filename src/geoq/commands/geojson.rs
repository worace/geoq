extern crate serde_json;
extern crate geojson;
extern crate clap;

use clap::ArgMatches;
use geojson::GeoJson;
use geoq::error::Error;
use geoq::reader::Reader;
use std::io;

use geoq::reader;


fn geom() -> Result<(), Error> {
    reader::for_entity(|e| {
        let gj_geom = e.geojson_geometry();
        println!("{}", serde_json::to_string(&gj_geom).unwrap());
        Ok(())
    })
}

fn feature() -> Result<(), Error> {
    reader::for_entity(|e| {
        let f = e.geojson_feature();
        println!("{}", serde_json::to_string(&f).unwrap());
        Ok(())
    })
}

fn feature_collection() -> Result<(), Error> {
    let mut features: Vec<geojson::Feature> = Vec::new();

    let stdin = io::stdin();
    let mut stdin_reader = stdin.lock();
    let reader = Reader::new(&mut stdin_reader);
    for e_res in reader {
        match e_res {
            Err(e) => return Err(e),
            Ok(e) => features.push(e.geojson_feature())
        }
    }

    let fc = geojson::FeatureCollection {
        bbox: None,
        features: features,
        foreign_members: None,
    };
    println!("{}", GeoJson::from(fc).to_string());
    Ok(())
}

pub fn run(gj: &ArgMatches) -> Result<(), Error> {
    match gj.subcommand() {
        ("geom", Some(_)) => geom(),
        ("f", Some(_)) => feature(),
        ("fc", Some(_)) => feature_collection(),
        _ => Err(Error::UnknownCommand),
    }
}
