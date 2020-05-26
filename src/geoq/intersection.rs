use geo_types::*;

use geo::algorithm::contains::Contains;
use geo::algorithm::intersects::Intersects;

pub fn poly_intersects(a: &Polygon<f64>, b: &Geometry<f64>) -> bool {
    match *b {
        Geometry::Point(ref point) => a.contains(point),
        Geometry::Line(ref line) => a.intersects(line),
        Geometry::LineString(ref ls) => a.intersects(ls),
        Geometry::Polygon(ref poly) => a.intersects(poly),
        Geometry::MultiPoint(ref mp) => mp.0.iter().any(|point| a.contains(point)),
        Geometry::MultiLineString(ref mls) => mls.0.iter().any(|ls| a.intersects(ls)),
        Geometry::MultiPolygon(ref mp) => mp.0.iter().any(|poly| a.intersects(poly)),
        Geometry::GeometryCollection(ref gc) => gc.0.iter().any(|geom| poly_intersects(a, geom)),
    }
}

pub fn multi_poly_intersects(a: &MultiPolygon<f64>, b: &Geometry<f64>) -> bool {
    a.0.iter().any(|poly| poly_intersects(poly, b))
}

pub fn line_intersects(a: &Line<f64>, b: &Geometry<f64>) -> bool {
    match *b {
        Geometry::Point(ref point) => a.intersects(point),
        Geometry::Line(ref line) => a.intersects(line),
        Geometry::LineString(ref ls) => a.intersects(ls),
        Geometry::Polygon(ref poly) => a.intersects(poly),
        Geometry::MultiPoint(ref mp) => mp.0.iter().any(|point| a.intersects(point)),
        Geometry::MultiLineString(ref mls) => mls.0.iter().any(|ls| a.intersects(ls)),
        Geometry::MultiPolygon(ref mp) => mp.0.iter().any(|poly| a.intersects(poly)),
        Geometry::GeometryCollection(ref gc) => gc.0.iter().any(|geom| line_intersects(a, geom)),
    }
}

pub fn linestring_intersects_point(a: &LineString<f64>, b: &Point<f64>) -> bool {
    a.lines().any(|line| line.intersects(b))
}

pub fn point_intersects(a: &Point<f64>, b: &Geometry<f64>) -> bool {
    match *b {
        Geometry::Point(ref point) => a == point,
        Geometry::Line(ref line) => a.intersects(line),
        Geometry::LineString(ref ls) => linestring_intersects_point(ls, a),
        Geometry::Polygon(ref poly) => poly.contains(a),
        Geometry::MultiPoint(ref mp) => mp.0.iter().any(|point| a == point),
        Geometry::MultiLineString(ref mls) => {
            mls.0.iter().any(|ls| linestring_intersects_point(ls, a))
        }
        Geometry::MultiPolygon(ref mp) => mp.0.iter().any(|poly| poly.contains(a)),
        Geometry::GeometryCollection(ref gc) => gc.0.iter().any(|geom| point_intersects(a, geom)),
    }
}

pub fn linestring_intersects(a: &LineString<f64>, b: &Geometry<f64>) -> bool {
    match *b {
        Geometry::Point(ref point) => linestring_intersects_point(a, point),
        Geometry::Line(ref line) => a.intersects(line),
        Geometry::LineString(ref ls) => a.intersects(ls),
        Geometry::Polygon(ref poly) => a.intersects(poly),
        Geometry::MultiPoint(ref mp) => {
            mp.0.iter()
                .any(|point| linestring_intersects_point(a, point))
        }
        Geometry::MultiLineString(ref mls) => mls.0.iter().any(|ls| a.intersects(ls)),
        Geometry::MultiPolygon(ref mp) => mp.0.iter().any(|poly| a.intersects(poly)),
        Geometry::GeometryCollection(ref gc) => {
            gc.0.iter().any(|geom| linestring_intersects(a, geom))
        }
    }
}

pub fn multi_point_intersects(a: &MultiPoint<f64>, b: &Geometry<f64>) -> bool {
    a.0.iter().any(|point| point_intersects(point, b))
}

pub fn multi_linestring_intersects(a: &MultiLineString<f64>, b: &Geometry<f64>) -> bool {
    a.0.iter().any(|ls| linestring_intersects(ls, b))
}

pub fn intersects(a: &Geometry<f64>, b: &Geometry<f64>) -> bool {
    match *a {
        Geometry::Point(ref g) => point_intersects(g, b),
        Geometry::Line(ref g) => line_intersects(g, b),
        Geometry::LineString(ref g) => linestring_intersects(g, b),
        Geometry::Polygon(ref g) => poly_intersects(g, b),
        Geometry::MultiPoint(ref g) => multi_point_intersects(g, b),
        Geometry::MultiLineString(ref g) => multi_linestring_intersects(g, b),
        Geometry::MultiPolygon(ref g) => multi_poly_intersects(g, b),
        Geometry::GeometryCollection(ref gc) => gc.0.iter().any(|geom| intersects(geom, b)),
    }
}
