extern crate geo_types;
extern crate geohash;
extern crate geojson;
extern crate wkt;
extern crate regex;

use geo_types::{Geometry, Point, Polygon, LineString};
use geojson::conversion::*;
use geojson::GeoJson;
use geoq::input::Input;
use geoq::error::Error;
use geoq::input;
use wkt::Wkt;
use wkt::ToGeo;
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

fn geohash_geom(raw: &String) -> geo_types::Geometry<f64> {
    let (bl, tr) = geohash::decode_bbox(raw);
    let outer = LineString(vec![
        Point::new(bl.x, bl.y),
        Point::new(tr.x, bl.y),
        Point::new(tr.x, tr.y),
        Point::new(bl.x, tr.y),
        Point::new(bl.x, bl.y),
    ]);
    Geometry::Polygon(Polygon::new(outer, Vec::new()))
}

fn wkt_entities(raw: &String) -> Vec<Entity> {
    let wkt_res: Result<Wkt<f64>, &str> = Wkt::from_str(raw);
    let mut entities = Vec::new();
    // TODO Handle invalid WKTs better
    for wkt_geom in wkt_res.unwrap().items {
        entities.push(Entity::Wkt(wkt_geom))
    }
    entities
}

fn geojson_entities(raw: &String) -> Vec<Entity> {
    if let Ok(gj) = raw.parse() {
        match gj {
            GeoJson::Geometry(gj_geom) => vec![Entity::GeoJsonGeometry(gj_geom)],
            GeoJson::Feature(gj_feature) => vec![Entity::GeoJsonFeature(gj_feature)],
            GeoJson::FeatureCollection(_fc) => vec![],
        }
    } else {
        vec![]
    }
}

impl Entity {
    pub fn geom(&self) -> geo_types::Geometry<f64> {
        match *self {
            Entity::LatLon(ref raw) => latlon_geom(raw),
            Entity::Geohash(ref raw) => geohash_geom(raw),
            Entity::Wkt(ref wkt_geom) => wkt_geom.to_geo().unwrap(),
            Entity::GeoJsonGeometry(ref gj_geom) => gj_geom.value.try_into().unwrap(),
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
        Input::LatLon(raw) => vec![Entity::LatLon(raw)],
        Input::Geohash(raw) => vec![Entity::Geohash(raw)],
        Input::WKT(raw) => wkt_entities(&raw),
        Input::GeoJSON(raw) => geojson_entities(&raw),
        _ => vec![],
    }
}

#[cfg(test)]
mod tests {
    extern crate serde_json;
    use geoq::entity;
    use geoq::input::Input;
    use geo_types::{Polygon, LineString, Point};

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

    #[test]
    fn entities_for_geohash() {
        let i = Input::Geohash("9q5".to_string());
        let entities = entity::from_input(i);

        let expected = Polygon::new(
            vec![
                [-119.53125, 33.75],
                [-118.125, 33.75],
                [-118.125, 35.15625],
                [-119.53125, 35.15625],
                [-119.53125, 33.75],
            ].into(),
            vec![],
        );
        let geom = entities[0].geom().as_polygon().unwrap();
        assert_eq!(expected, geom);

        let gj_geom = entities[0].geojson_geom();
        let geom_json = serde_json::to_string(&gj_geom).unwrap();
        let exp_json = "{\"coordinates\":[[[-119.53125,33.75],[-118.125,33.75],[-118.125,35.15625],[-119.53125,35.15625],[-119.53125,33.75]]],\"type\":\"Polygon\"}";
        assert_eq!(exp_json, geom_json);
    }

    #[test]
    fn entities_for_wkt() {
        let i = Input::WKT("LINESTRING (30 10, 10 30, 40 40)".to_string());
        let entities = entity::from_input(i);

        assert_eq!(1, entities.len());

        let expected = LineString(
            vec![
                Point::new(30.0, 10.0),
                Point::new(10.0, 30.0),
                Point::new(40.0, 40.0),
            ].into(),
        );
        let geom = entities[0].geom().as_linestring().unwrap();
        assert_eq!(expected, geom);

        let gj_geom = entities[0].geojson_geom();
        let geom_json = serde_json::to_string(&gj_geom).unwrap();
        let exp_json = "{\"coordinates\":[[30.0,10.0],[10.0,30.0],[40.0,40.0]],\"type\":\"LineString\"}";
        assert_eq!(exp_json, geom_json);
    }

    #[test]
    fn entities_for_geojson_geom() {
        let raw = "{\"type\": \"LineString\", \"coordinates\": [[-26.01, 59.17], [-15.46, 45.58], [0.35, 35.74]]}";
        let i = Input::GeoJSON(raw.to_string());
        let entities = entity::from_input(i);

        assert_eq!(1, entities.len());

        let expected = LineString(
            vec![
                Point::new(30.0, 10.0),
                Point::new(10.0, 30.0),
                Point::new(40.0, 40.0),
            ].into(),
        );
        let geom = entities[0].geom().as_linestring().unwrap();
        assert_eq!(expected, geom);

        let gj_geom = entities[0].geojson_geom();
        let geom_json = serde_json::to_string(&gj_geom).unwrap();
        let exp_json = "{\"coordinates\":[[30.0,10.0],[10.0,30.0],[40.0,40.0]],\"type\":\"LineString\"}";
        assert_eq!(exp_json, geom_json);
    }
}
