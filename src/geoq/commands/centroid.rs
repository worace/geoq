use crate::geoq::{self, error::Error, reader};
use geo_types::{Geometry, Point};
use geojson;
use serde_json;

fn gj_point(point: Point<f64>) -> String {
    let geom = Geometry::Point(point);
    let gj = geojson::Geometry::new(geojson::Value::from(&geom));
    serde_json::to_string(&gj).unwrap()
}

pub fn run() -> Result<(), Error> {
    reader::for_entity(|e| {
        let raw = e.raw();
        let g = e.geom();
        match geoq::centroid::centroid(&g) {
            Some(point) => println!("{}", gj_point(point)),
            None => eprintln!("Could not calculate centroid for geom: {}", raw),
        }
        Ok(())
    })
}
