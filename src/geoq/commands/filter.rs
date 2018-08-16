extern crate geo_types;

use geoq;
use clap::ArgMatches;
use geoq::error::Error;
use geoq::entity;
use geoq::reader;
use geo_types::{Geometry, Polygon};
use geoq::input;

fn intersects(matches: &ArgMatches) -> Result<(), Error> {
    match matches.value_of("query") {
        Some(q) => {
            let query_input = try!(input::read_line(q.to_string()));
            let query_entities = try!(entity::from_input(query_input));
            if query_entities.is_empty() {
                Err(Error::UnknownEntityFormat)
            } else {
                let query_geoms: Vec<Geometry<f64>> = query_entities.into_iter().map(|e| e.geom()).collect();

                reader::for_entity(|entity| {
                    let output = entity.raw();
                    let geom = entity.geom();
                    if query_geoms.iter().any(|ref query_geom| {
                        geoq::intersection::intersects(query_geom, &geom)
                    }) {
                        println!("{}", output);
                    }
                    Ok(())
                })
            }
        }
        _ => Err(Error::MissingArgument),
    }
}

pub fn run(matches: &ArgMatches) -> Result<(), Error> {
    match matches.subcommand() {
        ("intersects", Some(m)) => intersects(m),
        _ => Err(Error::UnknownCommand),
    }
}
