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
    println!("{:?}", closest);
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
    extern crate geo_types;
    extern crate geoq_wkt;

    use geoq::distance::distance;
    use geo_types::{Geometry, Point, Polygon};
    use geoq_wkt::ToWkt;

    #[test]
    #[ignore]
    fn test_point_to_point() {
        let la = Point::new(-118.2437, 34.0522);
        let ny = Point::new(-74.0060, 40.7128);
        let nyg = Geometry::Point(ny);

        // POLYGON((-119.49554443359376 33.58030298537655,-119.33898925781251 33.58030298537655,-119.33898925781251 33.667211101197545,-119.49554443359376 33.667211101197545,-119.49554443359376 33.58030298537655))
        // 3944422.23148992
        match distance(&la, &nyg) {
            Some(d) => assert_eq!(d.round(), 3944422.),
            None => assert!(false, "Should get distance")
        }

        // let dist = distance(&a, &b);
        // assert_eq!(Some(3944422), dist);
    }

    #[test]
    fn test_containing_polygon() {
        let la = Point::new(-118.2437, 34.0522);
        let poly = Polygon::new(
            vec![
                [-119.53125, 33.75],
                [-118.125, 33.75],
                [-118.125, 35.15625],
                [-119.53125, 35.15625],
                [-119.53125, 33.75],
            ].into(),
            vec![],
        );
        let polyg = Geometry::Polygon(poly);
        println!("{}", polyg.to_wkt().items.pop().unwrap());
        match distance(&la, &polyg) {
            // Currently finding closest point on perimeter of the polygon
            // instead of recognizing containment
            Some(d) => assert_eq!(d.round(), 10959.),
            // Some(d) => assert_eq!(d.round(), 0.),
            None => assert!(false, "Should get distance")
        }
    }
}
