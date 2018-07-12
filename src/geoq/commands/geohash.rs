extern crate geo_types;
extern crate geohash;

use geoq;
use geoq::reader;
use geoq::entity;
use geoq::input::Input;
use geoq::error::Error;
use clap::ArgMatches;

fn read_level(matches: &ArgMatches) -> Result<usize, Error> {
    let level_arg = matches.value_of("level");
    if level_arg.is_none() {
        return Err(Error::MissingArgument);
    }

    let level_str = level_arg.unwrap();

    let level_parsed = level_str.parse::<usize>();
    if level_parsed.is_err() {
        return Err(Error::InvalidNumberFormat)
    }
    Ok(level_parsed.unwrap())
}

fn point(matches: &ArgMatches) -> Result<(), Error> {
    let level = try!(read_level(matches));

    reader::for_entity(|e| {
        match e.geom() {
            geo_types::Geometry::Point(p) => {
                println!("{}", geohash::encode(p.0, level));
                Ok(())
            }
            _ => Err(Error::NotImplemented),
        }
    })
}

fn covering(matches: &ArgMatches) -> Result<(), Error> {
    let level = try!(read_level(matches));
    reader::for_input(|i| {
        if matches.is_present("original") {
            println!("{}", i.raw());
        }

        for entity in entity::from_input(i) {
            let g = entity.geom();
            for gh in geoq::geohash::covering(&g, level) {
                println!("{}", gh);
            }
        }
        Ok(())
    })
}

fn children() -> Result<(), Error> {
    reader::for_input(|i| {
        match i {
            Input::Geohash(ref raw) => {
                for gh in geoq::geohash::children(raw) {
                    println!("{}", gh);
                }
                Ok(())
            }
            _ => Err(Error::NotImplemented),
        }
    })
}

fn neighbors(matches: &ArgMatches) -> Result<(), Error> {
    let exclude = matches.is_present("exclude");
    reader::for_input(|i| {
        match i {
            Input::Geohash(ref raw) => {
                for gh in geoq::geohash::neighbors(raw, !exclude).iter() {
                    println!("{}", gh);
                }
                Ok(())
            }
            _ => Err(Error::NotImplemented),
        }
    })
}

pub fn run(matches: &ArgMatches) -> Result<(), Error> {
    match matches.subcommand() {
        ("point", Some(m)) => point(m),
        ("children", Some(_)) => children(),
        ("neighbors", Some(m)) => neighbors(m),
        ("covering", Some(m)) => covering(m),
        _ => Err(Error::UnknownCommand),
    }
}
