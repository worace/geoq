extern crate geo_types;
extern crate geo;

use geo_types::{Geometry, Polygon, LineString, Point, MultiPolygon};

use geo::algorithm::contains::Contains;
use geo::algorithm::intersects::Intersects;

pub fn poly_intersects(a: &Polygon<f64>, b: &Geometry<f64>) -> bool {
    match *b {
        Geometry::Point(ref point) => a.contains(point),
        Geometry::LineString(ref ls) => a.intersects(ls),
        Geometry::Polygon(ref poly) => a.intersects(poly),
        Geometry::MultiPolygon(ref mp) => {
            mp.0.iter().any(|poly| a.intersects(poly))
        },
        _ => false
    }
}

pub fn multi_poly_intersects(a: &MultiPolygon<f64>, b: &Geometry<f64>) -> bool {
    a.0.iter().any(|poly| poly_intersects(poly, b))
}

pub fn linestring_intersects_point(a: &LineString<f64>, b: &Point<f64>) -> bool {
    a.lines().any(|line| line.intersects(b))
}

pub fn point_intersects(a: &Point<f64>, b: &Geometry<f64>) -> bool {
    match *b {
        Geometry::Point(ref point) => a == point,
        Geometry::LineString(ref ls) => linestring_intersects_point(ls, a),
        Geometry::Polygon(ref poly) => poly.contains(a),
        Geometry::MultiPolygon(ref mp) => {
            mp.0.iter().any(|poly| poly.contains(a))
        },
        _ => false
    }
}

pub fn linestring_intersects(a: &LineString<f64>, b: &Geometry<f64>) -> bool {
    match *b {
        Geometry::Point(ref point) => linestring_intersects_point(a, point),
        Geometry::LineString(ref ls) => a.intersects(ls),
        Geometry::Polygon(ref poly) => a.intersects(poly),
        Geometry::MultiPolygon(ref mp) => {
            mp.0.iter().any(|poly| a.intersects(poly))
        },
        _ => false
    }
}

pub fn intersects(a: &Geometry<f64>, b: &Geometry<f64>) -> bool {
    match *a {
        Geometry::Point(ref g) => point_intersects(g, b),
        Geometry::LineString(ref g) => linestring_intersects(g, b),
        Geometry::Polygon(ref g) => poly_intersects(g, b),
        Geometry::MultiPolygon(ref g) => multi_poly_intersects(g, b),
        _ => false
    }
}
