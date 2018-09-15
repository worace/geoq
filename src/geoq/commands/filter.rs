use geoq;
use clap::ArgMatches;
use geoq::error::Error;
use geoq::entity;
use geo_types::{Geometry, Polygon};
use geoq::input;
use geoq::par;

fn intersects(matches: &ArgMatches) -> Result<(), Error> {
    match matches.value_of("query") {
        Some(q) => {
            let query_input = try!(input::read_line(q.to_string()));
            let query_entities = try!(entity::from_input(query_input));
            if query_entities.is_empty() {
                Err(Error::UnknownEntityFormat)
            } else {
                let query_geoms: Vec<Geometry<f64>> = query_entities.into_iter().map(|e| e.geom()).collect();

                par::for_stdin_entity(move |entity| {
                    let output = entity.raw();
                    let geom = entity.geom();
                    if query_geoms.iter().any(|ref query_geom| {
                        geoq::intersection::intersects(query_geom, &geom)
                    }) {
                        Ok(vec![output])
                    } else {
                        Ok(vec![])
                    }
                })
            }
        }
        _ => Err(Error::MissingArgument),
    }
}

fn contains(matches: &ArgMatches) -> Result<(), Error> {
    match matches.value_of("query") {
        Some(q) => {
            let query_input = try!(input::read_line(q.to_string()));
            let query_entities = try!(entity::from_input(query_input));
            if query_entities.is_empty() {
                Err(Error::UnknownEntityFormat)
            } else {
                let query_geoms: Vec<Geometry<f64>> = query_entities.into_iter().map(|e| e.geom()).collect();
                let query_polygons: Vec<Polygon<f64>> = query_geoms.into_iter().flat_map(|geom| {
                    match geom {
                        Geometry::Polygon(poly) => vec![poly],
                        Geometry::MultiPolygon(mp) => mp.0,
                        _ => vec![]
                    }
                }).collect();

                if query_polygons.is_empty() {
                    Err(Error::PolygonRequired)
                } else {
                    par::for_stdin_entity(move |entity| {
                        let output = entity.raw();
                        let geom = entity.geom();
                        if query_polygons.iter().any(|ref query_poly| {
                            geoq::contains::contains(query_poly, &geom)
                        }) {
                            Ok(vec![output])
                        } else {
                            Ok(vec![])
                        }
                    })
                }
            }
        }
        _ => Err(Error::MissingArgument),
    }
}

pub fn run(matches: &ArgMatches) -> Result<(), Error> {
    match matches.subcommand() {
        ("intersects", Some(m)) => intersects(m),
        ("contains", Some(m)) => contains(m),
        _ => Err(Error::UnknownCommand),
    }
}
