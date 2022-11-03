use crate::geoq::{self, entity::Entity, error::Error, par};
use clap::ArgMatches;
use geo::Point;
use h3ron::{FromH3Index, H3Cell, Index};
use std::{
    io::{self, prelude::*},
    str::FromStr,
};

fn read_resolution(matches: &ArgMatches) -> Result<u8, Error> {
    let resolution_arg = matches.value_of("resolution");
    if resolution_arg.is_none() {
        return Err(Error::MissingArgument);
    }

    let resolution_str = resolution_arg.unwrap();

    let resolution_parsed = resolution_str.parse::<u8>();
    if resolution_parsed.is_err() {
        return Err(Error::InvalidNumberFormat(format!(
            "Expected valid H3 cell resolution: {}",
            resolution_str
        )));
    }
    let resolution = resolution_parsed.unwrap();
    if resolution > 15 {
        return Err(Error::InvalidInput(format!(
            "Invalid H3 cell resolution: {}. Expected number from 0 to 15.",
            resolution_str
        )));
    }
    Ok(resolution)
}

fn point(matches: &ArgMatches) -> Result<(), Error> {
    let resolution = read_resolution(matches)?;

    par::for_stdin_entity(move |e| match e.geom() {
        geo_types::Geometry::Point(p) => cell_at_res(p, resolution).map(|c| vec![c.to_string()]),
        _ => Err(Error::InvalidInput(
            "Input for 'geoq h3 point' should be a Point geometry".to_string(),
        )),
    })
}

fn cell_children(cell: H3Cell, _res: Option<u8>) -> Result<Vec<String>, Error> {
    let cell_res = cell.resolution();
    if _res.filter(|r| r <= &cell_res).is_some() {
        return Err(Error::InvalidInput(format!(
            "Resolution must be greater than provided cell resolution."
        )));
    }
    let res: u8 = _res.unwrap_or_else(|| cell.resolution() + 1);
    if cell_res == 15 {
        return Err(Error::InvalidInput(format!(
            "Can't get children for resolution 15 cell. Cell: {}",
            cell.to_string()
        )));
    }
    cell.get_children(res)
        .map_err(|e| {
            Error::InvalidInput(format!(
                "Unable to compute children for H3 Cell: {} -- {}",
                cell.to_string(),
                e
            ))
        })
        .map(|cells| cells.iter().map(|c| c.to_string()).collect())
}

fn children(matches: &ArgMatches) -> Result<(), Error> {
    let resolution = match read_resolution(matches) {
        Ok(res) => Some(res),
        Err(Error::MissingArgument) => None,
        err => return err.map(|_| ()),
    };

    par::for_stdin_entity(move |e| match e {
        Entity::H3(cell) => cell_children(cell, resolution),
        _ => Err(Error::InvalidInput(format!(
            "Input for 'geoq h3 children' should be a hexadecimal h3 cell. Got: {}",
            e
        ))),
    })
}

fn cell_at_res(p: Point<f64>, res: u8) -> Result<H3Cell, Error> {
    H3Cell::from_point(p, res).map_err(|e| {
        Error::InvalidInput(format!(
            "Unable to calculate h3 cell for point {},{} -- {}",
            p.x(),
            p.y(),
            e
        ))
    })
}

fn hierarchy() -> Result<(), Error> {
    par::for_stdin_entity(move |e| match e.geom() {
        geo_types::Geometry::Point(p) => {
            let res: Result<Vec<String>, Error> = (0..=15)
                .map(|res| cell_at_res(p, res).map(|c| c.to_string()))
                .collect();
            res
        }
        _ => Err(Error::InvalidInput(
            "Input for 'geoq h3 hierarchy' should be a Point geometry".to_string(),
        )),
    })
}

fn from_str() -> Result<(), Error> {
    par::for_stdin_entity(move |e| match e {
        Entity::H3(cell) => Ok(vec![cell.h3index().to_string()]),
        other => Err(Error::InvalidInput(format!(
            "geoq h3 from-str requires H3 cell strings as inputs -- got {}",
            other
        ))),
    })
}

fn h3_from_int(i: u64) -> Result<H3Cell, Error> {
    let cell = H3Cell::from_h3index(i);
    if cell.is_valid() {
        Ok(cell)
    } else {
        Err(Error::InvalidInput(format!(
            "Invalid H3 Index: {} -- Is not a valid 64-bit integer H3 cell representation.",
            i
        )))
    }
}

// match l.parse::<u64>() {
//                Ok(gh_num) => println!("{}", geoq::geohash::encode_long(gh_num)),
//                _ => {
//                    return Err(Error::InvalidNumberFormat(format!(
//                        "Expected u64 geohash number: {}",
//                        l
//                    )))
//                }
//            }
fn to_str() -> Result<(), Error> {
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        match line {
            Ok(l) => {
                let res = l
                    .parse::<u64>()
                    .map_err(|_| {
                        Error::InvalidInput(format!(
                            "Expected valid H3 integer cell index. Got: {}",
                            l
                        ))
                    })
                    .and_then(h3_from_int);

                match res {
                    Ok(cell) => println!("{}", cell.to_string()),
                    e => return e.map(|_| ()),
                }
            }
            _ => return Err(Error::IOError),
        }
    }
    Ok(())
}

fn read_radius(matches: &ArgMatches) -> Result<Option<u32>, Error> {
    let arg = matches.value_of("radius");
    if arg.is_none() {
        return Ok(None);
    }

    let radius_str = arg.unwrap();

    radius_str
        .parse::<u32>()
        .map_err(|_| {
            Error::InvalidNumberFormat(format!("Expected integer radius, got: {}", radius_str))
        })
        .map(|radius| Some(radius))
}

fn cell_disk(cell: H3Cell, radius: u32) -> Result<Vec<String>, Error> {
    cell.grid_disk(radius)
        .map(|cells| cells.iter().map(|c| c.to_string()).collect())
        .map_err(|e| {
            Error::ProgramError(format!(
                "Unable to compute H3 grid disk for cell: {}, radius: {}.",
                cell.to_string(),
                radius
            ))
        })
}

fn grid_disk(matches: &ArgMatches) -> Result<(), Error> {
    let radius_opt = read_radius(matches)?;
    let radius = radius_opt.unwrap_or(1);

    par::for_stdin_entity(move |e| match e {
        Entity::H3(cell) => cell_disk(cell, radius),
        other => Err(Error::InvalidInput(format!(
            "geoq h3 grid-disk requires H3 cell strings as inputs -- got {}",
            other
        ))),
    })
}

// H3 methods
// [x] point
// [ ] polyfill
// [x] hierarchy -- print full hierarchy
// [x] children
// [x] disk (n)
// [x] string to long
// [x] long to string
// [ ] H3 metadata in geojson representation
// H3 add to entity parsing
// - long or hex format? will it collide with geohash?
pub fn run(matches: &ArgMatches) -> Result<(), Error> {
    match matches.subcommand() {
        ("point", Some(m)) => point(m),
        ("children", Some(m)) => children(m),
        ("hierarchy", _) => hierarchy(),
        ("from-str", _) => from_str(),
        ("to-str", _) => to_str(),
        ("grid-disk", Some(m)) => grid_disk(m),
        _ => Err(Error::UnknownCommand),
    }
}
