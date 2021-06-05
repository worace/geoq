use crate::geoq::{error::Error};
use clap::ArgMatches;
use shapefile;
use dbase::{FieldValue, Record};
use serde_json::{json, Map, Value, Number};
use geojson;

impl From<shapefile::Error> for Error {
    fn from(_err: shapefile::Error) -> Self {
        Error::ShapefileReaderError
    }
}

impl From<String> for Error {
    fn from(err: String) -> Self {
        Error::ProgramError(err)
    }
}

fn record_to_json(record: Record) -> Result<serde_json::Map<String, Value>, String> {
    let mut json = Map::new();
    for (name, value) in record.into_iter() {
        match value {
            FieldValue::Character(Some(string)) => {
                let json_str = serde_json::Value::String(string);
                json.insert(name, json_str);
            },
            FieldValue::Double(num) => {
                let num = Number::from_f64(num).ok_or("Could not convert shp number to JSON")?;
                let json_num = serde_json::Value::Number(num);
                json.insert(name, json_num);
            }
            _ => (),
        }
    };
    Ok(json)
}

fn shp_to_gj_geom(geom: shapefile::Shape) -> Result<geojson::Geometry, String> {
    match geom {
        shapefile::Shape::Point(point) => {
            let gj_point = geojson::Value::Point(vec![point.x, point.y]);
            let gj_geom = geojson::Geometry::new(gj_point);
            Ok(gj_geom)
        },
        _ => Err("Invalid shape type".to_string())
    }
}

fn shp_to_geojson(geom: shapefile::Shape, record: Record) -> Result<geojson::Feature, String> {
    let gj_geom = shp_to_gj_geom(geom)?;
    let props = record_to_json(record)?;
    Ok(geojson::Feature {
        id: None,
        bbox: None,
        foreign_members: None,
        geometry: Some(gj_geom),
        properties: Some(props)
    })
}


pub fn run(m: &ArgMatches) -> Result<(), Error> {
    let path = m.value_of("path").unwrap();
    let mut reader = shapefile::Reader::from_path(path)?;
    for shape_record in reader.iter_shapes_and_records() {
        // for each shape, match it by type and convert
        // to the appropriate geojson type
        // for each properties record, convert it to a JSON object
        // that will go into the GJ feature's properties field
        let (shape, record) = shape_record?;
        let gj = shp_to_geojson(shape, record)?;
        let str = serde_json::to_string(&gj).unwrap();
        println!("{}", str);
    }
    Ok(())
}
