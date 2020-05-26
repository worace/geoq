use crate::geoq::{self, entity::Entity, error::Error, par};
use clap::ArgMatches;
use std::io::{self, prelude::*};

fn read_level(matches: &ArgMatches) -> Result<usize, Error> {
    let level_arg = matches.value_of("level");
    if level_arg.is_none() {
        return Err(Error::MissingArgument);
    }

    let level_str = level_arg.unwrap();

    let level_parsed = level_str.parse::<usize>();
    if level_parsed.is_err() {
        return Err(Error::InvalidNumberFormat);
    }
    Ok(level_parsed.unwrap())
}

fn point(matches: &ArgMatches) -> Result<(), Error> {
    let level = read_level(matches)?;

    par::for_stdin_entity(move |e| match e.geom() {
        geo_types::Geometry::Point(p) => match geohash::encode(p.0, level) {
            Ok(gh) => Ok(vec![gh]),
            _ => Err(Error::InvalidGeohashPoint),
        },
        _ => Err(Error::NotImplemented),
    })
}

fn covering(matches: &ArgMatches) -> Result<(), Error> {
    let level = read_level(matches)?;
    let include_original = matches.is_present("original");
    par::for_stdin_entity(move |e| {
        if include_original {
            let mut results = vec![e.raw()];
            let g = e.geom();
            results.extend(geoq::geohash::covering(&g, level));
            Ok(results)
        } else {
            let g = e.geom();
            Ok(geoq::geohash::covering(&g, level))
        }
    })
}

fn children() -> Result<(), Error> {
    par::for_stdin_entity(|e| match e {
        Entity::Geohash(ref raw) => Ok(geoq::geohash::children(raw)),
        _ => Err(Error::NotImplemented),
    })
}

fn neighbors(matches: &ArgMatches) -> Result<(), Error> {
    let exclude = matches.is_present("exclude");
    par::for_stdin_entity(move |e| match e {
        Entity::Geohash(ref raw) => Ok(geoq::geohash::neighbors(raw, !exclude)),
        _ => Err(Error::NotImplemented),
    })
}

fn roots() -> Result<(), Error> {
    for c in geoq::geohash::BASE_32.iter() {
        println!("{}", c);
    }
    Ok(())
}

fn encode_long() -> Result<(), Error> {
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        match line {
            Ok(l) => match l.parse::<u64>() {
                Ok(gh_num) => println!("{}", geoq::geohash::encode_long(gh_num)),
                _ => return Err(Error::InvalidNumberFormat),
            },
            _ => return Err(Error::IOError),
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
        ("roots", Some(_)) => roots(),
        ("encode-long", Some(_)) => encode_long(),
        _ => Err(Error::UnknownCommand),
    }
}
