extern crate geo_types;
extern crate geohash;
extern crate geojson;
extern crate wkt;
extern crate regex;

use geo_types::{Geometry, Point};
use geojson::GeoJson;
use geoq::input::Input;
use geoq::error::Error;
use geoq::input;
use wkt::Wkt;
use regex::Regex;

lazy_static! {
    static ref LATLON_SPLIT: Regex = Regex::new(",|\t").unwrap();
}

pub enum Entity {
    LatLon(String),
    Geohash(String),
    Wkt(wkt::Geometry<f64>),
    GeoJsonFeature(geojson::Feature),
    GeoJsonGeometry(geojson::Geometry),
}

fn latlon_geom(raw: &String) -> geo_types::Geometry<f64> {
    let pieces = LATLON_SPLIT.split(raw).collect::<Vec<&str>>();
    let lat = pieces[0].parse::<f64>().unwrap();
    let lon = pieces[1].parse::<f64>().unwrap();
    Geometry::Point(Point::new(lon, lat))
}

impl Entity {
    pub fn geom(&self) -> geo_types::Geometry<f64> {
        match *self {
            Entity::LatLon(ref raw) => latlon_geom(raw),
            _ => Geometry::Point(Point::new(0.0, 0.0))
        }
    }

    pub fn geojson_geom(&self) -> geojson::Geometry {
        let geom = self.geom();
        geojson::Geometry::new(geojson::Value::from(&geom))
    }
}

pub fn from_input(i: Input) -> Vec<Entity> {
    match i {
        Input::LatLon(raw) => {
            vec![Entity::LatLon(raw)]
        },
        _ => vec![],
    }
}

#[cfg(test)]
mod tests {
    extern crate serde_json;
    use geoq::entity;
    use geoq::input::Input;

    #[test]
    fn entities_for_latlon() {
        let i = Input::LatLon("12,34".to_string());
        let entities = entity::from_input(i);
        assert_eq!(1, entities.len());
        let geom = entities[0].geom().as_point().unwrap();
        assert_eq!(12.0, geom.0.y);
        assert_eq!(34.0, geom.0.x);
        let gj_geom = entities[0].geojson_geom();
        let json = serde_json::to_string(&gj_geom).unwrap();
        assert_eq!("{\"coordinates\":[34.0,12.0],\"type\":\"Point\"}", json);
    }
}
