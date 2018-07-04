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
use serde_json;

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

fn parsed_geojson_entities(gj: GeoJson) -> Vec<Entity> {
    match gj {
        GeoJson::Geometry(gj_geom) => vec![Entity::GeoJsonGeometry(gj_geom)],
        GeoJson::Feature(gj_feature) => vec![Entity::GeoJsonFeature(gj_feature)],
        GeoJson::FeatureCollection(gj_fc) => {
            gj_fc
                .features
                .into_iter()
                .flat_map(|f| parsed_geojson_entities(GeoJson::Feature(f)))
                .collect()
        }
    }
}

fn geojson_entities(raw: &String) -> Vec<Entity> {
    if let Ok(gj) = raw.parse() {
        parsed_geojson_entities(gj)
    } else {
        eprintln!("****** FAILED TO PARSE GEOJSON");
        eprintln!("{}", raw);
        vec![]
    }
}

impl Entity {
    pub fn geom(self) -> geo_types::Geometry<f64> {
        match self {
            Entity::LatLon(ref raw) => latlon_geom(raw),
            Entity::Geohash(ref raw) => geohash_geom(raw),
            Entity::Wkt(ref wkt_geom) => wkt_geom.to_geo().unwrap(),
            Entity::GeoJsonGeometry(gj_geom) => gj_geom.value.try_into().unwrap(),
            Entity::GeoJsonFeature(gj_feature) => gj_feature.geometry.unwrap().value.try_into().unwrap(),
            _ => Geometry::Point(Point::new(0.0, 0.0))
        }
    }

    pub fn geojson_geometry(self) -> geojson::Geometry {
        let geom = self.geom();
        geojson::Geometry::new(geojson::Value::from(&geom))
    }

    pub fn geojson_properties(&self) -> serde_json::Map<String, serde_json::value::Value> {
        match *self {
            Entity::GeoJsonFeature(ref f) => {
                if let Some(props) = &f.properties {
                    props.clone()
                } else {
                    serde_json::Map::new()
                }
            }
            _ => serde_json::Map::new()
        }
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
        let entities = entity::from_input(i.clone());
        assert_eq!(1, entities.len());

        let geom = entity::from_input(i.clone()).pop().unwrap().geom().as_point().unwrap();
        assert_eq!(12.0, geom.0.y);
        assert_eq!(34.0, geom.0.x);

        let gj_geom = entity::from_input(i.clone()).pop().unwrap().geojson_geometry();
        let json = serde_json::to_string(&gj_geom).unwrap();
        assert_eq!("{\"coordinates\":[34.0,12.0],\"type\":\"Point\"}", json);

        assert_eq!(serde_json::Map::new(), entity::from_input(i.clone()).pop().unwrap().geojson_properties());
    }

    #[test]
    fn entities_for_geohash() {
        let i = Input::Geohash("9q5".to_string());
        let entities = entity::from_input(i.clone());

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
        let geom = entity::from_input(i.clone()).pop().unwrap().geom().as_polygon().unwrap();
        assert_eq!(expected, geom);

        let gj_geom = entity::from_input(i.clone()).pop().unwrap().geojson_geometry();
        let geom_json = serde_json::to_string(&gj_geom).unwrap();
        let exp_json = "{\"coordinates\":[[[-119.53125,33.75],[-118.125,33.75],[-118.125,35.15625],[-119.53125,35.15625],[-119.53125,33.75]]],\"type\":\"Polygon\"}";
        assert_eq!(exp_json, geom_json);

        assert_eq!(serde_json::Map::new(), entity::from_input(i.clone()).pop().unwrap().geojson_properties());
    }

    #[test]
    fn entities_for_wkt() {
        let i = Input::WKT("LINESTRING (30 10, 10 30, 40 40)".to_string());
        let entities = entity::from_input(i.clone());

        assert_eq!(1, entities.len());

        let expected = LineString(
            vec![
                Point::new(30.0, 10.0),
                Point::new(10.0, 30.0),
                Point::new(40.0, 40.0),
            ].into(),
        );
        let geom = entity::from_input(i.clone()).pop().unwrap().geom().as_linestring().unwrap();
        assert_eq!(expected, geom);

        let gj_geom = entity::from_input(i.clone()).pop().unwrap().geojson_geometry();
        let geom_json = serde_json::to_string(&gj_geom).unwrap();
        let exp_json = "{\"coordinates\":[[30.0,10.0],[10.0,30.0],[40.0,40.0]],\"type\":\"LineString\"}";
        assert_eq!(exp_json, geom_json);

        assert_eq!(serde_json::Map::new(), entity::from_input(i.clone()).pop().unwrap().geojson_properties());
    }

    #[test]
    fn entities_for_geojson_geometry() {
        let raw = "{\"type\": \"LineString\", \"coordinates\": [[-26.01, 59.17], [-15.46, 45.58], [0.35, 35.74]]}";
        let i = Input::GeoJSON(raw.to_string());
        let entities = entity::from_input(i.clone());

        assert_eq!(1, entities.len());

        let expected = LineString(
            vec![
                Point::new(-26.01, 59.17),
                Point::new(-15.46, 45.58),
                Point::new(0.35, 35.74),
            ].into(),
        );
        let geom = entity::from_input(i.clone()).pop().unwrap().geom().as_linestring().unwrap();
        assert_eq!(expected, geom);

        let gj_geom = entity::from_input(i.clone()).pop().unwrap().geojson_geometry();
        let geom_json = serde_json::to_string(&gj_geom).unwrap();
        let exp_json = "{\"coordinates\":[[-26.01,59.17],[-15.46,45.58],[0.35,35.74]],\"type\":\"LineString\"}";
        assert_eq!(exp_json, geom_json);

        assert_eq!(serde_json::Map::new(), entity::from_input(i.clone()).pop().unwrap().geojson_properties());
    }

    #[test]
    fn entities_for_geojson_feature() {
        // TODO - make properties map optional for geojson inputs?
        let raw = "{\"type\": \"Feature\", \"properties\": {\"pizza\": \"pie\"}, \"geometry\": {\"type\": \"LineString\", \"coordinates\": [[-26.01, 59.17], [-15.46, 45.58], [0.35, 35.74]]}}";
        let i = Input::GeoJSON(raw.to_string());
        let entities = entity::from_input(i.clone());

        assert_eq!(1, entities.len());

        let expected = LineString(
            vec![
                Point::new(-26.01, 59.17),
                Point::new(-15.46, 45.58),
                Point::new(0.35, 35.74),
            ].into(),
        );
        let geom = entity::from_input(i.clone()).pop().unwrap().geom().as_linestring().unwrap();
        assert_eq!(expected, geom);

        let gj_geom = entity::from_input(i.clone()).pop().unwrap().geojson_geometry();
        let geom_json = serde_json::to_string(&gj_geom).unwrap();
        let exp_json = "{\"coordinates\":[[-26.01,59.17],[-15.46,45.58],[0.35,35.74]],\"type\":\"LineString\"}";
        assert_eq!(exp_json, geom_json);

        let mut exp_properties = serde_json::Map::new();
        exp_properties.insert(
            String::from("pizza"),
            serde_json::to_value("pie").unwrap(),
        );
        assert_eq!(exp_properties, entity::from_input(i.clone()).pop().unwrap().geojson_properties());
    }

    #[test]
    fn entities_for_geojson_feature_collection() {
        let raw = r#"{"type":"FeatureCollection","features":[{"type":"Feature","properties":{"a":"b"},"geometry":{"type":"Point","coordinates":[34.0,12.0]}},{"type":"Feature","properties":{"c":1},"geometry":{"type":"Point","coordinates":[78.0,56.0]}}]}"#;
        let i = Input::GeoJSON(raw.to_string());
        let entities = entity::from_input(i.clone());

        assert_eq!(2, entities.len());

        let expected = vec![
            Point::new(34.0, 12.0),
            Point::new(78.0, 56.0),
        ];
        let geoms: Vec<Point<f64>> = entity::from_input(i.clone())
            .into_iter()
            .map(|e| e.geom().as_point().expect("Should parse to points"))
            .collect();
        assert_eq!(expected, geoms);

        let expected_json = vec![
            "{\"coordinates\":[34.0,12.0],\"type\":\"Point\"}",
            "{\"coordinates\":[78.0,56.0],\"type\":\"Point\"}"
        ];
        let json: Vec<String> = entity::from_input(i.clone())
            .into_iter()
            .map(|e| e.geojson_geometry())
            .map(|j| serde_json::to_string(&j).unwrap())
            .collect();
        assert_eq!(expected_json, json);

        let mut props1 = serde_json::Map::new();
        props1.insert(String::from("a"), serde_json::to_value("b").unwrap());
        let mut props2 = serde_json::Map::new();
        props2.insert(String::from("c"), serde_json::to_value(1).unwrap());

        let props: Vec<serde_json::Map<String, serde_json::value::Value>> = entity::from_input(i.clone())
            .into_iter()
            .map(|e| e.geojson_properties())
            .collect();
        assert_eq!(vec![props1, props2], props);
    }
}
