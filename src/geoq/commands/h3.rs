use crate::geoq::{self, entity::Entity, error::Error, par};
use clap::ArgMatches;
use h3ron::H3Cell;
use std::io::{self, prelude::*};

fn read_level(matches: &ArgMatches) -> Result<u8, Error> {
    let level_arg = matches.value_of("resolution");
    if level_arg.is_none() {
        return Err(Error::MissingArgument);
    }

    let level_str = level_arg.unwrap();

    let level_parsed = level_str.parse::<u8>();
    if level_parsed.is_err() {
        return Err(Error::InvalidNumberFormat(format!(
            "Expected valid H3 cell resolution: {}",
            level_str
        )));
    }
    let level = level_parsed.unwrap();
    if level > 15 {
        return Err(Error::InvalidInput(format!(
            "Invalid H3 cell resolution: {}. Expected number from 0 to 15.",
            level_str
        )));
    }
    Ok(level)
}

fn point(matches: &ArgMatches) -> Result<(), Error> {
    let level = read_level(matches)?;

    par::for_stdin_entity(move |e| match e.geom() {
        geo_types::Geometry::Point(p) => match H3Cell::from_point(p, level) {
            Ok(cell) => Ok(vec![cell.to_string()]),
            Err(e) => Err(Error::InvalidInput(format!(
                "Unable to calculate h3 cell for point {},{} -- {}",
                p.x(),
                p.y(),
                e
            ))),
        },
        _ => Err(Error::InvalidInput(
            "Input for 'geoq h3 point' should be a Point geometry".to_string(),
        )),
    })
}

// H3 methods
// point
// polyfill
// hierarchy -- print full hierarchy
// children
// disk (n)
// string to long
// long to string
// H3 add to entity parsing
// - long or hex format? will it collide with geohash?
pub fn run(matches: &ArgMatches) -> Result<(), Error> {
    match matches.subcommand() {
        ("point", Some(m)) => point(m),
        _ => Err(Error::UnknownCommand),
    }
}
