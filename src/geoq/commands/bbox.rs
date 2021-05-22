use crate::geoq::{bbox::BBoxToPoly, error::Error, par};

// Options
// --fold (combine all into 1 bbox)
// --embed (embed as bbox member of geojson feature)
pub fn run() -> Result<(), Error> {
    par::for_stdin_entity(|e| {
        let bbox = e.bbox();
        let poly = bbox.to_polygon();
        let gj = geojson::Geometry::new(geojson::Value::from(&poly));
        Ok(vec![format!("{}", serde_json::to_string(&gj).unwrap())])
    })
}
