use std::fmt::Debug;
use geo_types::*;
use std::cmp::Ordering;

// TODO Handle additional Multi* geometry types
trait OptRectHelper: Debug {
    fn or_zero(self) -> geo::Rect<f64>;
}

impl OptRectHelper for Option<geo::Rect<f64>> {
    fn or_zero(self) -> geo::Rect<f64> {
        self.unwrap_or_else(|| {
            let p = geo::Coordinate {x: 0.0, y: 0.0};
            geo::Rect {min: p, max: p}
        })
    }
}

fn rect(p: &Point<f64>) -> geo::Rect<f64> {
    let c = geo::Coordinate {x: p.x(), y: p.y()};
    geo::Rect {min: c, max: c}
}

fn min(a: f64, b: f64) -> f64 {
    match a.partial_cmp(&b).unwrap_or(Ordering::Equal) {
        Ordering::Less => a,
        Ordering::Equal => a,
        Ordering::Greater => b
    }
}

fn max(a: f64, b: f64) -> f64 {
    match a.partial_cmp(&b).unwrap_or(Ordering::Equal) {
        Ordering::Less => b,
        Ordering::Equal => a,
        Ordering::Greater => a
    }
}

fn merge(a: geo::Rect<f64>, b: geo::Rect<f64>) -> geo::Rect<f64> {
    let min = geo::Coordinate {
        x: min(a.min.x, b.min.x),
        y: min(a.min.y, b.min.y)
    };
    let max = geo::Coordinate {
        x: max(a.max.x, b.max.x),
        y: max(a.max.y, b.max.y)
    };
    geo::Rect{min: min, max: max}
}

pub fn bbox(geom: &Geometry<f64>) -> geo::Rect<f64> {
    use geo::algorithm::bounding_rect::BoundingRect;
    match *geom {
        Geometry::Point(ref g) => rect(g),
        Geometry::Line(ref g) => g.bounding_rect(),
        Geometry::LineString(ref g) => g.bounding_rect().or_zero(),
        Geometry::Polygon(ref g) => g.bounding_rect().or_zero(),
        Geometry::MultiPoint(ref g) => g.bounding_rect().or_zero(),
        Geometry::MultiLineString(ref g) => g.bounding_rect().or_zero(),
        Geometry::MultiPolygon(ref g) => g.bounding_rect().or_zero(),
        Geometry::GeometryCollection(ref gc) => {
            let rects: Vec<geo::Rect<f64>> = gc.0.iter().map(|geom| bbox(geom)).collect();

            rects.iter()
                .reduce(|a, b| merge(a, b))
                .or_zero()
        }
    }
}
