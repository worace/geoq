extern crate geo_types;

use geoq;
use clap::ArgMatches;
use geoq::error::Error;
use geoq::entity;
use geoq::reader;
use geo_types::{Geometry, Polygon};
use geoq::input;
use std::io;

fn intersects(matches: &ArgMatches) -> Result<(), Error> {
    match matches.value_of("query") {
        Some(q) => {
            let query_input = try!(input::read_line(q.to_string()));
            let query_entities = entity::from_input(query_input);
            if query_entities.is_empty() {
                Err(Error::UnknownEntityFormat)
            } else {
                let query_geoms = query_entities.into_iter().map(|e| e.geom());
                let query_polygons: Vec<Polygon<f64>> = query_geoms
                    .flat_map(|g| match g {
                        Geometry::Polygon(p) => vec![p],
                        Geometry::MultiPolygon(mp) => mp.0,
                        _ => vec![],
                    })
                    .collect();


                reader::for_input(|input| {
                    // TODO restructure so this doesnlt need to be cloned
                    let output = input.raw().clone();
                    let entities = entity::from_input(input);
                    let geoms: Vec<Geometry<f64>> =
                        entities.into_iter().map(|e| e.geom()).collect();
                    if query_polygons.iter().any(|ref query_poly| {
                        geoms
                            .iter()
                            .any(|ref e_geom| geoq::geohash::intersects(query_poly, e_geom))
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
