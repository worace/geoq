use crate::geoq::error::Error;
use geo_types::{Geometry, Point};
use geojson::GeoJson;
use clap::ArgMatches;
use serde_json;
use serde_json::{json, Map, Value};
use std::{convert::TryInto};
use std::io::{self, BufRead};

pub fn find_number(v: &Map<String, Value>, keys: &Vec<&'static str>) -> Option<(&'static str, f64)> {
    for k in keys {
        if !v.contains_key(*k) {
            continue;
        }
        match v[*k] {
            Value::Number(ref n) => return n.as_f64().map(|f| (*k, f)),
            Value::String(ref s) => return s.parse::<f64>().ok().map(|f| (*k, f)),
            _ => continue,
        }
    }
    None
}

pub fn find_string(v: &Map<String, Value>, keys: &Vec<&'static str>) -> Option<(&'static str, String)> {
    for k in keys {
        if !v.contains_key(*k) {
            continue;
        }
        match v[*k] {
            Value::String(ref s) => return Some((*k, s.to_string())),
            _ => continue,
        }
    }
    None
}

pub fn find_object(v: &Map<String, Value>, keys: &Vec<&'static str>) -> Option<(&'static str, Map<String, Value>)> {
    for k in keys {
        if !v.contains_key(*k) {
            continue;
        }
        match v[*k] {
            Value::Object(ref obj) => return Some((*k, obj.clone())),
            _ => continue,
        }
    }
    None
}

type Geom = Geometry<f64>;

fn latlon_point(v: &Map<String, Value>) -> Option<(Geom, Vec<&'static str>)> {
    let lat = find_number(&v, &vec!["latitude", "lat"]);
    let lon = find_number(&v, &vec!["longitude", "lon", "lng"]);
    let lat_lon: Option<_> = try { (lat?, lon?) };
    lat_lon.map(|((lat_key, lat), (lon_key, lon))| (Geometry::Point(Point::new(lon, lat)), vec![lat_key, lon_key]))
}

fn wkt_geom(v: &Map<String, Value>) -> Option<(Geom, Vec<&'static str>)> {
    let str_opt_with_key = find_string(v, &vec!["geometry", "wkt"]);
    str_opt_with_key.and_then(|(k, v)| wkt::Wkt::from_str(&v).ok().map(|wkt| (k, wkt)))
        .and_then(|(k,wkt)| {
            if wkt.items.is_empty() {
                None
            } else {
                // TODO what to do with multiple wkt geoms
                wkt::conversion::try_into_geometry(&wkt.items[0]).ok().map(|geom| (geom, vec![k]))
            }
        })
}

fn geojson_str_geom(v: &Map<String, Value>) -> Option<(Geom, Vec<&'static str>)> {
    let str_opt_with_key = find_string(v, &vec!["geometry", "geojson"]);
    str_opt_with_key.and_then(|(k, v)| v.parse().ok().map(|gj| (k, gj)))
        .and_then(|(k,gj)| {
            match gj {
                GeoJson::Geometry(gj_geom) => TryInto::<Geom>::try_into(gj_geom.value).ok().map(|geom|(geom, vec![k])),
                _ => None
            }
        })
}

fn geojson_geom(v: &Map<String, Value>) -> Option<(Geom, Vec<&'static str>)> {
    let json_opt_with_key = find_object(v, &vec!["geometry", "geojson"]);
    json_opt_with_key.and_then(|(k, v)| geojson::Geometry::from_json_object(v).ok().map(|gj_geom| (k, gj_geom)))
        .and_then(|(k,gj_geom)| {
            TryInto::<Geom>::try_into(gj_geom.value).ok().map(|geom|(geom, vec![k]))
        })
}

pub fn find_geometry(v: &Map<String, Value>) -> Option<(Geom, Vec<&'static str>)> {
    latlon_point(v).or_else(|| wkt_geom(v)).or_else(|| geojson_str_geom(v)).or_else(|| geojson_geom(v))
    // latlon_point(v).or().or().or()
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
            Value::Object(mut o) => {
                match find_geometry(&o) {
                    Some((geom, geomified_keys)) => {
                        for k in geomified_keys {
                            o.remove(k);
                        }
                        let gj_geom = geojson::Geometry::new(geojson::Value::from(&geom));
                        let geojson = json!({
                            "type": "Feature",
                            "properties": Value::Object(o),
                            "geometry": gj_geom
                        });
                        let json_str = serde_json::to_string(&geojson)?;
                        println!("{}", json_str)
                    },
                    _ => {
                        eprintln!("Couldn't guess GeoJSON Feature from JSON");
                        return Err(Error::InvalidJSONType);
                    }
                }
            },
            _ => return Err(Error::InvalidJSONType),
        }
    }
    Ok(())
}

pub fn run(m: &ArgMatches) -> Result<(), Error> {
    match m.subcommand() {
        ("munge", Some(_)) => munge(),
        _ => Err(Error::UnknownCommand),
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};
    use geo_types::{Geometry, Point};

    use crate::geoq::commands::json::find_geometry;

    fn check_geom(v: Value) -> Option<(Geometry<f64>, Vec<&'static str>)> {
        find_geometry(v.as_object().expect("Require object"))
    }

    fn point(lat: f64, lon: f64) -> Geometry<f64> {
        Geometry::Point(Point::new(lon, lat))
    }

    #[test]
    fn lat_lon_combos() {
        assert_eq!(Some((point(1.0, 2.0), vec!["lat", "lon"])), check_geom(json!({"lat": 1.0, "lon": 2.0,})));
        assert_eq!(Some((point(1.0, 2.0), vec!["lat", "longitude"])), check_geom(json!({"lat": 1.0, "longitude": 2.0,})));
        assert_eq!(Some((point(1.0, 2.0), vec!["latitude", "longitude"])), check_geom(json!({"latitude": 1.0, "longitude": 2.0,})));
        assert_eq!(Some((point(1.0, 2.0), vec!["latitude", "longitude"])), check_geom(json!({"latitude": "1.0", "longitude": "2.0",})));
    }

    #[test]
    fn wkt() {
        assert_eq!(Some((point(1.0, 2.0), vec!["wkt"])), check_geom(json!({"wkt": "POINT (2.0 1.0)"})));
        assert_eq!(Some((point(1.0, 2.0), vec!["geometry"])), check_geom(json!({"geometry": "POINT (2.0 1.0)"})));
    }

    #[test]
    fn geojson_string() {
        assert_eq!(Some((point(1.0, 2.0), vec!["geojson"])), check_geom(json!({"geojson": "{\"type\": \"Point\", \"coordinates\": [2.0, 1.0]}"})));
        assert_eq!(Some((point(1.0, 2.0), vec!["geometry"])), check_geom(json!({"geometry": "{\"type\": \"Point\", \"coordinates\": [2.0, 1.0]}"})));
    }

    #[test]
    fn geojson_object() {
        assert_eq!(Some((point(1.0, 2.0), vec!["geojson"])), check_geom(json!({"geojson": {"type": "Point", "coordinates": [2.0, 1.0]}})));
        assert_eq!(Some((point(1.0, 2.0), vec!["geometry"])), check_geom(json!({"geometry": {"type": "Point", "coordinates": [2.0, 1.0]}})));
    }
}
