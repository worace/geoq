use crate::geoq::{error::Error, par, reader::Reader};
use clap::ArgMatches;
use geojson::GeoJson;
use std::io;

fn geom() -> Result<(), Error> {
    par::for_stdin_entity(|e| {
        let gj_geom = e.geojson_geometry();
        Ok(vec![serde_json::to_string(&gj_geom).unwrap()])
    })
}

fn feature() -> Result<(), Error> {
    par::for_stdin_entity(|e| {
        let f = e.geojson_feature();
        Ok(vec![serde_json::to_string(&f).unwrap()])
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
            Ok(e) => features.push(e.geojson_feature()),
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
