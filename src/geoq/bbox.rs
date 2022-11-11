use geo_types::*;
use std::cmp::Ordering;
use std::fmt::Debug;

trait OptRectHelper: Debug {
    fn or_zero(self) -> geo::Rect<f64>;
}

pub fn zero_rect() -> geo::Rect<f64> {
    let p = geo::Coordinate { x: 0.0, y: 0.0 };
    geo::Rect::new(p, p)
}

impl OptRectHelper for Option<geo::Rect<f64>> {
    fn or_zero(self) -> geo::Rect<f64> {
        self.unwrap_or_else(|| zero_rect())
    }
}

fn rect(p: &Point<f64>) -> geo::Rect<f64> {
    let c = geo::Coordinate { x: p.x(), y: p.y() };
    geo::Rect::new(c, c)
}

fn min(a: f64, b: f64) -> f64 {
    match a.partial_cmp(&b).unwrap_or(Ordering::Equal) {
        Ordering::Less => a,
        Ordering::Equal => a,
        Ordering::Greater => b,
    }
}

fn max(a: f64, b: f64) -> f64 {
    match a.partial_cmp(&b).unwrap_or(Ordering::Equal) {
        Ordering::Less => b,
        Ordering::Equal => a,
        Ordering::Greater => a,
    }
}

pub fn merge(a: &geo::Rect<f64>, b: &geo::Rect<f64>) -> geo::Rect<f64> {
    let min = geo::Coordinate {
        x: min(a.min().x, b.min().x),
        y: min(a.min().y, b.min().y),
    };
    let max = geo::Coordinate {
        x: max(a.max().x, b.max().x),
        y: max(a.max().y, b.max().y),
    };
    geo::Rect::new(min, max)
}

pub fn bbox(geom: &Geometry<f64>) -> geo::Rect<f64> {
    use geo::algorithm::bounding_rect::BoundingRect;
    match *geom {
        Geometry::Point(ref g) => rect(g),
        Geometry::Line(ref g) => g.bounding_rect(),
        Geometry::LineString(ref g) => g.bounding_rect().or_zero(),
        Geometry::Polygon(ref g) => g.bounding_rect().or_zero(),
        Geometry::Rect(ref g) => g.to_polygon().bounding_rect().or_zero(),
        Geometry::Triangle(ref g) => g.to_polygon().bounding_rect().or_zero(),
        Geometry::MultiPoint(ref g) => g.bounding_rect().or_zero(),
        Geometry::MultiLineString(ref g) => g.bounding_rect().or_zero(),
        Geometry::MultiPolygon(ref g) => g.bounding_rect().or_zero(),
        Geometry::GeometryCollection(ref gc) => {
            let rects: Vec<geo::Rect<f64>> = gc.0.iter().map(|geom| bbox(geom)).collect();

            rects.iter().fold(zero_rect(), |a, b| merge(&a, b))
        }
    }
}

pub trait BBoxToPoly {
    fn to_polygon_geoq(&self) -> geo_types::Polygon<f64>;
}

impl BBoxToPoly for geo::Rect<f64> {
    fn to_polygon_geoq(&self) -> geo_types::Polygon<f64> {
        Polygon::new(
            LineString::from(vec![
                self.max().x_y(),
                (self.min().x, self.max().y),
                (self.min().x, self.min().y),
                (self.max().x, self.min().y),
                self.max().x_y(),
            ]),
            vec![],
        )
    }
}
