use crate::geoq::error::Error;
use geo_types::{Geometry, Point};
use clap::ArgMatches;
use serde_json::{json, Map, Number, Value};
use std::io::{self, BufRead};

pub fn find_number(v: &Map<String, Value>, keys: &Vec<&'static str>) -> Option<(&'static str, f64)> {
    for k in keys {
        if !v.contains_key(*k) {
            continue;
        }
        match v[*k] {
            Value::Number(ref n) => return n.as_f64().map(|f| (*k, f)),
            _ => continue,
        }
    }
    None
}

type Geom = Geometry<f64>;

fn latlon_point(v: &Map<String, Value>) -> Option<(Geom, Vec<String>)> {
    let lat = find_number(&v, &vec!["latitude", "lat"]);
    let lon = find_number(&v, &vec!["longitude", "lon", "lng"]);
    let lat_lon: Option<_> = try { (lat?, lon?) };
    lat_lon.map(|((lat_key, lat), (lon_key, lon))| (Geometry::Point(Point::new(lon, lat)), vec![lat_key.to_string(), lon_key.to_string()]))
}

fn wkt_geom(v: &Map<String, Value>) -> Option<(Geom, Vec<String>)> {
    let geom_opt = v.get("geometry").map(|s| ("geometry", s));
    let wkt_opt = v.get("wkt").map(|s|("wkt", s));
    let str_opt = geom_opt.or(wkt_opt);
    str_opt.and_then(|(k, v)| v.as_str().map(|v| (k, v)))
        .and_then(|(k, v)| wkt::Wkt::from_str(v).ok().map(|wkt| (k, wkt)))
        .and_then(|(k,wkt)| {
            if (wkt.items.is_empty()) {
                None
            } else {
                wkt::conversion::try_into_geometry(&wkt.items[0]).ok().map(|geom| (geom, vec![k.to_string()]))
            }
        })
}

pub fn find_geometry(v: &Map<String, Value>) -> Option<(Geom, Vec<String>)> {
    latlon_point(v).or(wkt_geom(v))
    // latlon_point
    // Point
    // - lat/lon
    // - lat/lng
    // - latitude/longitude
    // Geometry
    // - wkt
    // - geometry: geojson string
    // - geometry: geojson geometry
    // None
}

// fn point() -> Result<(), Error> {
//     let stdin = io::stdin();
//     for l in stdin.lock().lines() {
//         let line = l?;
//         let v: Value = serde_json::from_str(&line)?;
//         match v {
//             Value::Object(o) => {
//                 match (
//                     find_number(&o, &vec!["latitude", "lat"]),
//                     find_number(&o, &vec!["longitude", "lon", "lng"]),
//                 ) {
//                     (Some(lat), Some(lon)) => {
//                         let geojson = json!({
//                             "type": "Feature",
//                             "properties": Value::Object(o),
//                             "geometry": {
//                                 "type": "Point",
//                                 "coordinates": vec![Value::Number(lon), Value::Number(lat)]
//                             }
//                         });
//                         let json_str = serde_json::to_string(&geojson)?;
//                         println!("{}", json_str)
//                     }
//                     _ => return Err(Error::InvalidJSONType),
//                 }
//             }
//             _ => return Err(Error::InvalidJSONType),
//         }
//     }
//     Ok(())
// }

fn munge() -> Result<(), Error> {
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
                                "type": "Point"
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
        // ("point", Some(_)) => point(),
        ("munge", Some(_)) => munge(),
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
