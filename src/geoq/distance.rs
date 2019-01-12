extern crate geo_types;
extern crate geo;

use geo_types::{Geometry, Point};
use geo::algorithm::closest_point::ClosestPoint;
use geo::algorithm::vincenty_distance::VincentyDistance;

// TODO nearest point for other geom types

fn closest_point(a: &Point<f64>, b: &Geometry<f64>) -> geo::Closest<f64> {
    match *b {
        Geometry::Point(ref g) => g.closest_point(a),
        Geometry::Line(ref g) => g.closest_point(a),
        Geometry::LineString(ref g) => g.closest_point(a),
        Geometry::Polygon(ref g) => g.closest_point(a),
        Geometry::MultiPoint(ref g) => g.closest_point(a),
        Geometry::MultiLineString(ref g) => g.closest_point(a),
        Geometry::MultiPolygon(ref g) => g.closest_point(a),
        Geometry::GeometryCollection(ref gc) => {
            geo::Closest::SinglePoint(Point::new(0.0, 0.0))
            // 0.0
            // let distances = gc.0.iter().map(|geom| distance(a, geom));
            // let sorted = distances.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Equal));
            // sorted[0]
        }
    }
}

pub fn distance(a: &Point<f64>, b: &Geometry<f64>) -> Option<f64> {
    // get closest point if possible
    // get distance for that point...
    let closest = closest_point(a, b);
    match closest {
        geo::Closest::Intersection(_) => Some(0.0),
        geo::Closest::SinglePoint(p) => {
            let vin = p.vincenty_distance(a);
            match vin {
                Ok(d) => Some(d),
                Err(_) => None
            }
        },
        geo::Closest::Indeterminate => {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use geoq::distance::distance;
    extern crate geo_types;
    use geo_types::{Geometry, Point};

    #[test]
    fn test_point_to_point() {
        let a = Point::new(0.0, 0.0);
        let b = Geometry::Point(Point::new(1.0, 1.0));

        let dist = distance(&a, &b);
        assert_eq!(Some(156899.56829129544), dist);
    }
}
