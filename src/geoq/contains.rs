use geo::algorithm::contains::Contains;
use geo_types::{Geometry, Polygon};

pub fn contains(outer: &Polygon<f64>, inner: &Geometry<f64>) -> bool {
    match *inner {
        Geometry::Point(ref g) => outer.contains(g),
        Geometry::Line(ref g) => outer.contains(g),
        Geometry::LineString(ref g) => outer.contains(g),
        Geometry::Polygon(ref g) => outer.contains(g),
        Geometry::Rect(ref g) => outer.contains(&g.to_polygon()),
        Geometry::Triangle(ref g) => outer.contains(&g.to_polygon()),
        Geometry::MultiPoint(ref mp) => mp.0.iter().all(|p| outer.contains(p)),
        Geometry::MultiLineString(ref mls) => mls.0.iter().all(|ls| outer.contains(ls)),
        Geometry::MultiPolygon(ref mp) => mp.0.iter().all(|poly| outer.contains(poly)),
        Geometry::GeometryCollection(ref gc) => gc.0.iter().all(|geom| contains(outer, geom)),
    }
}

pub fn contains_any(outer: &Geometry<f64>, inner: &Geometry<f64>) -> bool {
    match *outer {
        Geometry::Point(ref g) => false,
        Geometry::Line(ref g) => false,
        Geometry::LineString(ref g) => false,
        Geometry::Polygon(ref g) => contains(g, inner),
        Geometry::Rect(ref g) => contains(&g.to_polygon(), inner),
        Geometry::Triangle(ref g) => contains(&g.to_polygon(), inner),
        Geometry::MultiPoint(ref mp) => false,
        Geometry::MultiLineString(ref mls) => false,
        Geometry::MultiPolygon(ref mp) => mp.0.iter().any(|poly| contains(poly, inner)),
        Geometry::GeometryCollection(ref gc) => gc.0.iter().any(|geom| contains_any(geom, inner)),
    }
}
