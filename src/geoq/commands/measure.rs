use crate::geoq::{distance, entity, error::Error, input, par};
use clap::ArgMatches;
use geo_types::Geometry;

fn distance(matches: &ArgMatches) -> Result<(), Error> {
    match matches.value_of("query") {
        Some(q) => {
            let query_input = input::read_line(q.to_string())?;
            let mut query_entities = entity::from_input(query_input)?;
            if query_entities.is_empty() {
                Err(Error::UnknownEntityFormat)
            } else if query_entities.len() > 1 {
                Err(Error::TooManyFeatures)
            } else {
                match query_entities.remove(0).geom() {
                    Geometry::Point(query_point) => par::for_stdin_entity(move |entity| {
                        let output = entity.raw();
                        let geom = entity.geom();

                        let dist = distance::distance(&query_point, &geom);

                        match dist {
                            Some(d) => Ok(vec![format!("{}\t{}", d, output)]),
                            None => {
                                eprintln!("Couldn't calculate distance between <query>: {:?} and <input>: {}", query_point, output);
                                Err(Error::DistanceFailed)
                            }
                        }
                    }),
                    _ => {
                        eprintln!(
                            "<query> argument for measuing distance must be a Point. Got: {}",
                            q
                        );
                        Err(Error::PointRequired)
                    }
                }
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
