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

fn intersects(matches: &ArgMatches, negate: bool) -> Result<(), Error> {
    use geo::algorithm::intersects::Intersects;
    let query_geoms = read_query_geoms(matches)?;
    par::for_stdin_entity(move |entity| {
        let output = entity.raw();
        let geom = entity.geom();
        let is_match: bool = query_geoms
            .iter()
            .any(|ref query_geom| query_geom.intersects(&geom));
        if is_match ^ negate {
            Ok(vec![output])
        } else {
            Ok(vec![])
        }
    })
}

fn contains(matches: &ArgMatches, negate: bool) -> Result<(), Error> {
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
            let is_match = query_polygons
                .iter()
                .any(|ref query_poly| geoq::contains::contains(query_poly, &geom));
            if is_match ^ negate {
                Ok(vec![output])
            } else {
                Ok(vec![])
            }
        })
    }
}

fn dwithin(matches: &ArgMatches, negate: bool) -> Result<(), Error> {
    let query_geoms = read_query_geoms(matches)?;
    let rad_arg = matches.value_of("radius").unwrap();
    let radius: f64 = rad_arg
        .parse()
        .map_err(|_| Error::InvalidNumberFormat(format!("Invalid Radius: {}", rad_arg)))?;

    if query_geoms.is_empty() {
        Err(Error::NoInputGiven)
    } else {
        par::for_stdin_entity(move |entity| {
            let output = entity.raw();
            let geom = entity.geom();
            let point = match geom {
                Geometry::Point(p) => Ok(p),
                _ => Err(Error::PointRequired),
            }?;
            let is_match = query_geoms.iter().any(|ref query_geom| {
                let dist = geoq::distance::distance(&point, query_geom);
                match dist {
                    Some(d) => d < radius,
                    None => false,
                }
            });
            if is_match ^ negate {
                Ok(vec![output])
            } else {
                Ok(vec![])
            }
        })
    }
}

pub fn run(matches: &ArgMatches) -> Result<(), Error> {
    // allow --negate to be passed either before or after the subcommand
    // geoq filter --negate intersects
    // OR
    // geoq filter intersects --negate
    let negate = matches
        .args
        .get("negate")
        .or(matches.subcommand().1.and_then(|m| (*m).args.get("negate")))
        .is_some();

    match matches.subcommand() {
        ("intersects", Some(m)) => intersects(m, negate),
        ("contains", Some(m)) => contains(m, negate),
        ("dwithin", Some(m)) => dwithin(m, negate),
        _ => Err(Error::UnknownCommand),
    }
}
