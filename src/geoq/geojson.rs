use geojson::GeoJson;
use serde_json::json;

// test helper fn
pub fn fvec(gj: &str) -> Vec<geojson::Feature> {
    use serde_json::json;
    let feat: GeoJson = gj.parse().expect("invalid geojson");

    match feat {
        GeoJson::Geometry(geom) => vec![geojson::Feature {
            geometry: Some(geom),
            bbox: None,
            foreign_members: None,
            id: None,
            properties: Some(serde_json::Map::new()),
        }],
        GeoJson::Feature(f) => vec![f],
        GeoJson::FeatureCollection(fc) => fc.features,
    }
}
