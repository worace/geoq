use clap::ArgMatches;
use geoq::error::Error;
use geoq::entity;
use geo_types::Geometry;
use geoq::input;
use geoq::par;

fn distance(matches: &ArgMatches) -> Result<(), Error> {
    match matches.value_of("query") {
        Some(q) => {
            let query_input = try!(input::read_line(q.to_string()));
            let mut query_entities = try!(entity::from_input(query_input));
            if query_entities.is_empty() {
                Err(Error::UnknownEntityFormat)
            } else if query_entities.len() > 1 {
                Err(Error::TooManyFeatures)
            } else {
                // let query_geoms: Vec<Geometry<f64>> = query_entities.into_iter().map(|e| e.geom()).collect();
                let query_geom: Geometry<f64> = query_entities.remove(0).geom();

                par::for_stdin_entity(move |entity| {
                    let output = entity.raw();
                    let geom = entity.geom();

                    println!("{:?}", query_geom);
                    println!("{:?}", geom);
                    Ok(vec![])
                    // if query_geoms.iter().any(|ref query_geom| {
                    //     geoq::intersection::intersects(query_geom, &geom)
                    // }) {
                    //     Ok(vec![output])
                    // } else {
                    //     Ok(vec![])
                    // }
                })
            }
        }
        _ => Err(Error::MissingArgument),
    }
}

pub fn run(matches: &ArgMatches) -> Result<(), Error> {
    match matches.subcommand() {
        ("distance", Some(m)) => distance(m),
        _ => Err(Error::UnknownCommand),
    }
}
