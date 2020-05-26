use crate::geoq::error::Error;
use geo_types::Geometry;
use geo_types::Point;
use serde_json;

fn get_ip_geolocation() -> Result<(), Error> {
    let resp = reqwest::get("http://ip-api.com/json");
    if let Err(e) = resp {
        eprintln!("Error fetching IP geolocation: {:?}", e);
        return Err(Error::IPGeolocationError);
    }

    let body = resp.unwrap().text();

    if let Err(e) = body {
        eprintln!("Error reading geolocation response: {:?}", e);
        return Err(Error::IPGeolocationError);
    }

    let json_res = serde_json::from_str(&body.unwrap());

    if let Err(e) = json_res {
        eprintln!("Error reading geolocation response: {:?}", e);
        return Err(Error::IPGeolocationError);
    }

    let json: serde_json::Value = json_res.unwrap();
    let lat = json["lat"].as_f64();
    let lon = json["lon"].as_f64();

    match (lat, lon) {
        (Some(lat), Some(lon)) => {
            let point = Geometry::Point(Point::new(lon, lat));
            let gj_point = geojson::Geometry::new(geojson::Value::from(&point));
            let geojson = serde_json::to_string(&gj_point).unwrap();
            println!("{}", geojson);
        }
        _ => eprintln!("Invalid IP location response: {}", json),
    }
    Ok(())
}

pub fn run() -> Result<(), Error> {
    get_ip_geolocation()
}
