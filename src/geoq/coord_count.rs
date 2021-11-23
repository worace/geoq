use geo_types::{Geometry, Polygon};

fn poly_coord_count(poly: &Polygon<f64>) -> usize {
    poly.exterior().num_coords()
        + poly
            .interiors()
            .iter()
            .map(|ring| ring.num_coords())
            .sum::<usize>()
}

pub fn coord_count(geom: &Geometry<f64>) -> usize {
    match *geom {
        Geometry::Point(_) => 1,
        Geometry::Line(_) => 2,
        Geometry::Triangle(_) => 3,
        Geometry::Rect(_) => 4,
        Geometry::LineString(ref g) => g.num_coords(),
        Geometry::Polygon(ref g) => poly_coord_count(g),
        Geometry::MultiPoint(ref g) => g.0.len(),
        Geometry::MultiLineString(ref g) => g.0.iter().map(|l| l.num_coords()).sum(),
        Geometry::MultiPolygon(ref g) => g.0.iter().map(|p| poly_coord_count(p)).sum(),
        Geometry::GeometryCollection(ref gc) => gc.0.iter().map(|g| coord_count(g)).sum(),
    }
}
