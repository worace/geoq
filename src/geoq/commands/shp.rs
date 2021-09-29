use crate::geoq::error::Error;
use clap::ArgMatches;
use dbase::{FieldValue, Record};
use geojson;
use serde_json::{Map, Number, Value};
use shapefile;

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
            }
            FieldValue::Double(num) => {
                let num = Number::from_f64(num).ok_or("Could not convert shp number to JSON")?;
                let json_num = serde_json::Value::Number(num);
                json.insert(name, json_num);
            }
            _ => (),
        }
    }
    Ok(json)
}

trait Pointable {
    fn vec(&self) -> Vec<f64>;
    fn gj_point(&self) -> geojson::Value {
        geojson::Value::Point(self.vec())
    }
    fn gj_geom(&self) -> geojson::Geometry {
        geojson::Geometry::new(geojson::Value::Point(self.vec()))
    }
    fn gj_geom_res(&self) -> Result<geojson::Geometry, String> {
        Ok(self.gj_geom())
    }
}

impl Pointable for shapefile::Point {
    fn vec(&self) -> Vec<f64> {
        vec![self.x, self.y]
    }
}
impl Pointable for shapefile::PointM {
    fn vec(&self) -> Vec<f64> {
        vec![self.x, self.y, self.m]
    }
}
impl Pointable for shapefile::PointZ {
    fn vec(&self) -> Vec<f64> {
        vec![self.x, self.y, self.m, self.z]
    }
}

trait PointIterable {
    fn vec(&self) -> Vec<Vec<Vec<f64>>>;
}

impl PointIterable for shapefile::Polyline {
    fn vec(&self) -> Vec<Vec<Vec<f64>>> {
        self.parts()
            .into_iter()
            .map(|part| part.into_iter().map(|p| p.vec()).collect::<Vec<Vec<f64>>>())
            .collect()
    }
}
impl PointIterable for shapefile::PolylineZ {
    fn vec(&self) -> Vec<Vec<Vec<f64>>> {
        self.parts()
            .into_iter()
            .map(|part| part.into_iter().map(|p| p.vec()).collect::<Vec<Vec<f64>>>())
            .collect()
    }
}
impl PointIterable for shapefile::PolylineM {
    fn vec(&self) -> Vec<Vec<Vec<f64>>> {
        self.parts()
            .into_iter()
            .map(|part| part.into_iter().map(|p| p.vec()).collect::<Vec<Vec<f64>>>())
            .collect()
    }
}

impl PointIterable for shapefile::Polygon {
    fn vec(&self) -> Vec<Vec<Vec<f64>>> {
        self.rings()
            .into_iter()
            .map(|r| {
                let ring: &shapefile::PolygonRing<shapefile::Point> = r;
                let point_vecs: Vec<Vec<f64>> =
                    ring.points().into_iter().map(|p| p.vec()).collect();
                point_vecs
            })
            .collect()
    }
}
impl PointIterable for shapefile::PolygonZ {
    fn vec(&self) -> Vec<Vec<Vec<f64>>> {
        self.rings()
            .into_iter()
            .map(|r| {
                let ring: &shapefile::PolygonRing<shapefile::PointZ> = r;
                let point_vecs: Vec<Vec<f64>> =
                    ring.points().into_iter().map(|p| p.vec()).collect();
                point_vecs
            })
            .collect()
    }
}
impl PointIterable for shapefile::PolygonM {
    fn vec(&self) -> Vec<Vec<Vec<f64>>> {
        self.rings()
            .into_iter()
            .map(|r| {
                let ring: &shapefile::PolygonRing<shapefile::PointM> = r;
                let point_vecs: Vec<Vec<f64>> =
                    ring.points().into_iter().map(|p| p.vec()).collect();
                point_vecs
            })
            .collect()
    }
}

// fn shp_to_gj_point(p: shapefile::Shape::PointZ)

fn shp_to_gj_geom(geom: shapefile::Shape) -> Result<geojson::Geometry, String> {
    match geom {
        shapefile::Shape::Point(g) => g.gj_geom_res(),
        shapefile::Shape::PointM(g) => g.gj_geom_res(),
        shapefile::Shape::PointZ(g) => g.gj_geom_res(),
        shapefile::Shape::Polyline(g) => Ok(geojson::Geometry::new(
            geojson::Value::MultiLineString(g.vec()),
        )),
        shapefile::Shape::PolylineZ(g) => Ok(geojson::Geometry::new(
            geojson::Value::MultiLineString(g.vec()),
        )),
        shapefile::Shape::PolylineM(g) => Ok(geojson::Geometry::new(
            geojson::Value::MultiLineString(g.vec()),
        )),
        shapefile::Shape::Polygon(g) => {
            Ok(geojson::Geometry::new(geojson::Value::Polygon(g.vec())))
        }
        shapefile::Shape::PolygonZ(g) => {
            Ok(geojson::Geometry::new(geojson::Value::Polygon(g.vec())))
        }
        shapefile::Shape::PolygonM(g) => {
            Ok(geojson::Geometry::new(geojson::Value::Polygon(g.vec())))
        }
        shapefile::Shape::Multipoint(g) => {
            let points: Vec<Vec<f64>> = g.points().into_iter().map(|p| p.vec()).collect();
            Ok(geojson::Geometry::new(geojson::Value::MultiPoint(points)))
        }
        shapefile::Shape::MultipointZ(g) => {
            let points: Vec<Vec<f64>> = g.points().into_iter().map(|p| p.vec()).collect();
            Ok(geojson::Geometry::new(geojson::Value::MultiPoint(points)))
        }
        shapefile::Shape::MultipointM(g) => {
            let points: Vec<Vec<f64>> = g.points().into_iter().map(|p| p.vec()).collect();
            Ok(geojson::Geometry::new(geojson::Value::MultiPoint(points)))
        }
        shapefile::Shape::NullShape => Ok(geojson::Geometry::new(geojson::Value::Polygon(vec![]))),
        shapefile::Shape::Multipatch(g) => {
            // This almost certainly does not work but it might be structurally valid ¯\_(ツ)_/¯
            let poly_vecs: Vec<Vec<Vec<f64>>> = g
                .patches()
                .into_iter()
                .map(|patch| {
                    patch
                        .points()
                        .into_iter()
                        .map(|point| point.vec())
                        .collect::<Vec<Vec<f64>>>()
                })
                .collect();
            Ok(geojson::Geometry::new(geojson::Value::Polygon(poly_vecs)))
        }
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
        properties: Some(props),
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
