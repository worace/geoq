use geo::algorithm::centroid::Centroid;
use geo_types::*;

// TODO Handle additional Multi* geometry types
pub fn centroid(geom: &Geometry<f64>) -> Option<Point<f64>> {
    match *geom {
        Geometry::Point(ref g) => Some(g.centroid()),
        Geometry::Line(ref g) => Some(g.centroid()),
        Geometry::LineString(ref g) => g.centroid(),
        Geometry::Polygon(ref g) => g.centroid(),
        // Geometry::MultiPoint(ref g) => g.centroid(),
        // Geometry::MultiLineString(ref g) => g.centroid(),
        Geometry::MultiPolygon(ref g) => g.centroid(),
        // Geometry::GeometryCollection(ref gc) => g.centroid()
        _ => None
    }
}
