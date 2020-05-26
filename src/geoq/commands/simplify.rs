use crate::geoq::{error::Error, par, simplify};
use clap::ArgMatches;
use std::str::FromStr;

fn simplify(epsilon: f64) -> Result<(), Error> {
    par::for_stdin_entity(move |e| {
        let props = e.geojson_properties();
        let simplified = simplify::simplify(e.geom(), epsilon);
        let gj_geom = geojson::Geometry::new(geojson::Value::from(&simplified));
        let feature = geojson::Feature {
            bbox: None,
            geometry: Some(gj_geom),
            id: None,
            properties: Some(props),
            foreign_members: None,
        };
        Ok(vec![serde_json::to_string(&feature).unwrap()])
    })
}

pub fn run(matches: &ArgMatches) -> Result<(), Error> {
    match matches.value_of("epsilon") {
        Some(arg) => match f64::from_str(arg) {
            Ok(eps) => simplify(eps),
            Err(_) => {
                eprintln!(
                    "Invalid Epsilon: {:?} - must be floating point number, e.g. 0.001.",
                    arg
                );
                Err(Error::InvalidNumberFormat)
            }
        },
        _ => Err(Error::MissingArgument),
    }
}
