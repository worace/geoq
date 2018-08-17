use geo::algorithm::contains::Contains;
use geo_types::{Geometry, Polygon};

pub fn contains(outer: &Polygon<f64>, inner: &Geometry<f64>) -> bool {
    match *inner {
        Geometry::Point(ref g) => outer.contains(g),
        Geometry::Line(ref g) => outer.contains(g),
        Geometry::LineString(ref g) => outer.contains(g),
        Geometry::Polygon(ref g) => outer.contains(g),
        Geometry::MultiPoint(ref mp) => mp.0.iter().all(|p| outer.contains(p)),
        Geometry::MultiLineString(ref mls) => mls.0.iter().all(|ls| outer.contains(ls)),
        Geometry::MultiPolygon(ref mp) => mp.0.iter().all(|poly| outer.contains(poly)),
        Geometry::GeometryCollection(ref gc) => gc.0.iter().all(|geom| contains(outer, geom))
    }
}
