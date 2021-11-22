use crate::geoq::{coord_count, error::Error, par, simplify};
use clap::ArgMatches;
use std::str::FromStr;

const MAX_ITERS: i32 = 20;

fn simplify(epsilon: f64, coords_target: Option<usize>) -> Result<(), Error> {
    par::for_stdin_entity(move |e| {
        let props = e.geojson_properties();
        let geom = e.geom();
        let simplified = match coords_target {
            None => simplify::simplify(geom, epsilon),
            Some(target) => {
                if coord_count::coord_count(&geom) <= target {
                    geom
                } else {
                    let mut eps = epsilon;
                    let mut simp = geom;
                    let mut iters = 0;
                    while coord_count::coord_count(&simp) > target && iters < MAX_ITERS {
                        simp = simplify::simplify(simp, eps);
                        eps = eps * 2.0;
                        iters += 1;
                    }
                    simp
                }
            }
        };

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
    let eps = match matches.value_of("epsilon") {
        Some(arg) => match f64::from_str(arg) {
            Ok(eps) => Ok(eps),
            Err(_) => {
                eprintln!(
                    "Invalid Epsilon: {:?} - must be floating point number, e.g. 0.001.",
                    arg
                );
                Err(Error::InvalidNumberFormat)
            }
        },
        _ => Err(Error::MissingArgument),
    };

    let target = match matches.value_of("to_coord_count") {
        Some(target) => {
            let parsed = target.parse::<usize>();
            if parsed.is_err() {
                eprintln!(
                    "Invalid to-size coordinate value: {:?} - must be a positive integer",
                    target
                );
                None
            } else {
                parsed.ok()
            }
        }
        _ => None,
    };

    eps.and_then(|eps| simplify(eps, target))
}
