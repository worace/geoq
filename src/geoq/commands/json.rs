use crate::geoq::error::Error;
use clap::ArgMatches;
use serde_json::{json, Map, Number, Value};
use std::io::{self, BufRead};

pub fn find_number(v: &Map<String, Value>, keys: &Vec<&str>) -> Option<Number> {
    for k in keys {
        if !v.contains_key(*k) {
            continue;
        }
        match v[*k] {
            Value::Number(ref n) => return Some(n.clone()),
            _ => continue,
        }
    }
    None
}

fn point() -> Result<(), Error> {
    let stdin = io::stdin();
    for l in stdin.lock().lines() {
        let line = l?;
        let v: Value = serde_json::from_str(&line)?;
        match v {
            Value::Object(o) => {
                match (
                    find_number(&o, &vec!["latitude", "lat"]),
                    find_number(&o, &vec!["longitude", "lon", "lng"]),
                ) {
                    (Some(lat), Some(lon)) => {
                        let geojson = json!({
                            "type": "Feature",
                            "properties": Value::Object(o),
                            "geometry": {
                                "type": "Point",
                                "coordinates": vec![Value::Number(lon), Value::Number(lat)]
                            }
                        });
                        let json_str = serde_json::to_string(&geojson)?;
                        println!("{}", json_str)
                    }
                    _ => return Err(Error::InvalidJSONType),
                }
            }
            _ => return Err(Error::InvalidJSONType),
        }
    }
    Ok(())
}

pub fn run(m: &ArgMatches) -> Result<(), Error> {
    match m.subcommand() {
        ("point", Some(_)) => point(),
        _ => Err(Error::UnknownCommand),
    }
}

// pub fn command() -> App {
//     let point: App = SubCommand::with_name("point")
//         .about("Attempt to convert arbitrary JSON into a GeoJSON point by checking for common latitude and longitude property names.");
//     SubCommand::with_name("json")
//         .about("Attempt to convert arbitrary geo-oriented JSON into GeoJSON")
//         .subcommand(point)
// }
