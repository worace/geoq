use geo::algorithm::closest_point::ClosestPoint;
use geo::algorithm::contains::Contains;
use geo::algorithm::vincenty_distance::VincentyDistance;
use geo_types::{Geometry, GeometryCollection, MultiPolygon, Point, Polygon};
use std::cmp::Ordering::Equal;

// TODO nearest point for other geom types

fn closest_point_to_poly(point: &Point<f64>, poly: &Polygon<f64>) -> geo::Closest<f64> {
    if poly.contains(point) {
        geo::Closest::Intersection(point.clone())
    } else {
        poly.closest_point(point)
    }
}

fn closest_point_to_multipoly(point: &Point<f64>, mp: &MultiPolygon<f64>) -> geo::Closest<f64> {
    if mp.contains(point) {
        geo::Closest::Intersection(point.clone())
    } else {
        mp.closest_point(point)
    }
}

fn closest_point_to_geom_coll(
    point: &Point<f64>,
    gc: &GeometryCollection<f64>,
) -> geo::Closest<f64> {
    if gc.0.len() == 0 {
        return geo::Closest::Indeterminate;
    }

    let mut measurements: Vec<(geo::Closest<f64>, f64)> =
        gc.0.iter()
            .map(|geom| closest_point(point, geom))
            .map(|closest| match closest {
                geo::Closest::Indeterminate => (closest, std::f64::INFINITY),
                geo::Closest::Intersection(_) => (closest, 0.0),
                geo::Closest::SinglePoint(p) => (
                    closest,
                    p.vincenty_distance(point).unwrap_or(std::f64::INFINITY),
                ),
            })
            .collect();

    measurements.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(Equal));

    measurements.remove(0).0
}

fn closest_point(a: &Point<f64>, b: &Geometry<f64>) -> geo::Closest<f64> {
    match *b {
        Geometry::Point(ref g) => g.closest_point(a),
        Geometry::Line(ref g) => g.closest_point(a),
        Geometry::LineString(ref g) => g.closest_point(a),
        Geometry::Polygon(ref g) => closest_point_to_poly(a, g),
        Geometry::MultiPoint(ref g) => g.closest_point(a),
        Geometry::MultiLineString(ref g) => g.closest_point(a),
        Geometry::MultiPolygon(ref g) => closest_point_to_multipoly(a, g),
        Geometry::GeometryCollection(ref gc) => closest_point_to_geom_coll(a, gc),
    }
}

pub fn distance(a: &Point<f64>, b: &Geometry<f64>) -> Option<f64> {
    let closest = closest_point(a, b);
    match closest {
        geo::Closest::Intersection(_) => Some(0.0),
        geo::Closest::SinglePoint(p) => {
            let vin = p.vincenty_distance(a);
            match vin {
                Ok(d) => Some(d),
                Err(_) => None,
            }
        }
        geo::Closest::Indeterminate => None,
    }
}

#[cfg(test)]
mod tests {
    use crate::geoq::distance::distance;
    use geo_types::{Geometry, Point, Polygon};
    use wkt::ToWkt;

    #[test]
    fn test_point_to_point() {
        let la = Point::new(-118.2437, 34.0522);
        let ny = Point::new(-74.0060, 40.7128);
        let nyg = Geometry::Point(ny);

        match distance(&la, &nyg) {
            Some(d) => assert_eq!(d.round(), 3944422.),
            None => assert!(false, "Should get distance"),
        }
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
            ]
            .into(),
            vec![],
        );
        let polyg = Geometry::Polygon(poly);
        println!("{}", polyg.to_wkt().items.pop().unwrap());
        match distance(&la, &polyg) {
            Some(d) => assert_eq!(d.round(), 0.),
            None => assert!(false, "Should get distance"),
        }
    }
}
