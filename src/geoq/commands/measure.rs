use crate::geoq::{coord_count, distance, entity, error::Error, input, par};
use clap::ArgMatches;
use geo_types::Geometry;
use serde_json::json;

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

fn coords(matches: &ArgMatches) -> Result<(), Error> {
    let gj = matches.is_present("geojson");
    par::for_stdin_entity(move |e| {
        let dupe = e.clone();
        let geom = e.geom();
        let count = coord_count::coord_count(&geom);
        if gj {
            let mut feature = dupe.geojson_feature();
            match feature.properties.as_mut() {
                Some(props) => {
                    props.insert("coord_count".to_string(), json!(count));
                }
                None => (),
            }
            Ok(vec![serde_json::to_string(&feature).unwrap()])
        } else {
            Ok(vec![format!("{}", count)])
        }
    })
}

pub fn run(matches: &ArgMatches) -> Result<(), Error> {
    match matches.subcommand() {
        ("distance", Some(m)) => distance(m),
        ("coord-count", Some(m)) => coords(m),
        _ => Err(Error::UnknownCommand),
    }
}
