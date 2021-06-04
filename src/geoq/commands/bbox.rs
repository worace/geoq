use crate::geoq::{error::Error, par, bbox::BBoxToPoly, reader, bbox};
use clap::ArgMatches;
use geo_types::Rect;

pub fn run(matches: &ArgMatches) -> Result<(), Error> {
    let embed = matches.is_present("embed");
    let all = matches.is_present("all");

    if all {
        let mut bbox: Option<Rect<f64>> = None;

        reader::for_entity(|e| {
            match bbox {
                Some(curr) => {
                    bbox = Some(bbox::merge(&curr, &e.bbox()));
                }
                None => {
                    bbox = Some(e.bbox());
                }
            }
            Ok(())
        })?;

        match bbox {
            None => {
                Err(Error::NoInputGiven)
            }
            Some(bbox) => {
                let poly = bbox.to_polygon();
                let gj = geojson::Geometry::new(geojson::Value::from(&poly));
                println!("{}", serde_json::to_string(&gj).unwrap());
                Ok(())
            }
        }
    } else {
        par::for_stdin_entity(move |e| {
            let bbox: Rect<f64> = e.bbox();

            if embed {
                let mut feat = e.geojson_feature();
                let gj_bbox = vec![bbox.min.x, bbox.min.y, bbox.max.x, bbox.max.y];
                feat.bbox = Some(gj_bbox);
                Ok(vec![format!("{}", serde_json::to_string(&feat).unwrap())])
            } else {
                let poly = bbox.to_polygon();
                let gj = geojson::Geometry::new(geojson::Value::from(&poly));
                Ok(vec![format!("{}", serde_json::to_string(&gj).unwrap())])
            }
        })
    }
}
