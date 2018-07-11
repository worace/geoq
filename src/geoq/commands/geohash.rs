extern crate geo_types;
extern crate geohash;

use geoq;
use geoq::reader::Reader;
use geoq::entity;
use geoq::input::Input;
use geoq::error::Error;
use clap::ArgMatches;
use std::io;

fn point(matches: &ArgMatches) -> Result<(), Error> {
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

fn covering(matches: &ArgMatches) -> Result<(), Error> {
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

fn children() -> Result<(), Error> {
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

fn neighbors(matches: &ArgMatches) -> Result<(), Error> {
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

pub fn run(matches: &ArgMatches) -> Result<(), Error> {
    match matches.subcommand() {
        ("point", Some(m)) => point(m),
        ("children", Some(_)) => children(),
        ("neighbors", Some(m)) => neighbors(m),
        ("covering", Some(m)) => covering(m),
        _ => Err(Error::UnknownCommand),
    }
}
