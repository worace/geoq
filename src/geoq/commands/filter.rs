use crate::geoq::{self, error::Error, par, reader::Reader};
use clap::ArgMatches;
use geo_types::{Geometry, Polygon};
use std::{fs::File, io::BufReader};

fn read_query_geoms(matches: &ArgMatches) -> Result<Vec<Geometry<f64>>, Error> {
    let f = matches.value_of("query-file");
    let q = matches.value_of("query");
    match (f, q) {
        (Some(path), None) => {
            let f = File::open(path)?;
            let mut f = BufReader::new(f);
            let reader = Reader::new(&mut f);
            let entities = reader.into_iter().collect::<Result<Vec<_>, _>>()?;
            Ok(entities.into_iter().map(|e| e.geom()).collect())
        }
        (None, Some(q)) => {
            let q_buff = q.as_bytes();
            let mut f = BufReader::new(q_buff);
            let reader = Reader::new(&mut f);
            let entities = reader.into_iter().collect::<Result<Vec<_>, _>>()?;
            Ok(entities.into_iter().map(|e| e.geom()).collect())
        }
        _ => {
            eprintln!("Must provide Query Features as either --file or positional argument.");
            Err(Error::MissingArgument)
        }
    }
}

fn intersects(matches: &ArgMatches) -> Result<(), Error> {
    let query_geoms = read_query_geoms(matches)?;
    par::for_stdin_entity(move |entity| {
        let output = entity.raw();
        let geom = entity.geom();
        if query_geoms
            .iter()
            .any(|ref query_geom| geoq::intersection::intersects(query_geom, &geom))
        {
            Ok(vec![output])
        } else {
            Ok(vec![])
        }
    })
}

fn contains(matches: &ArgMatches) -> Result<(), Error> {
    let query_geoms = read_query_geoms(matches)?;
    let query_polygons: Vec<Polygon<f64>> = query_geoms
        .into_iter()
        .flat_map(|geom| match geom {
            Geometry::Polygon(poly) => vec![poly],
            Geometry::MultiPolygon(mp) => mp.0,
            _ => vec![],
        })
        .collect();

    if query_polygons.is_empty() {
        Err(Error::PolygonRequired)
    } else {
        par::for_stdin_entity(move |entity| {
            let output = entity.raw();
            let geom = entity.geom();
            if query_polygons
                .iter()
                .any(|ref query_poly| geoq::contains::contains(query_poly, &geom))
            {
                Ok(vec![output])
            } else {
                Ok(vec![])
            }
        })
    }
}

pub fn run(matches: &ArgMatches) -> Result<(), Error> {
    match matches.subcommand() {
        ("intersects", Some(m)) => intersects(m),
        ("contains", Some(m)) => contains(m),
        _ => Err(Error::UnknownCommand),
    }
}
