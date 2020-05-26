use geo::algorithm::simplifyvw::SimplifyVWPreserve;
use geo_types::Geometry;

pub fn simplify(geom: Geometry<f64>, epsilon: f64) -> Geometry<f64> {
    match geom {
        Geometry::LineString(g) => Geometry::LineString(g.simplifyvw_preserve(&epsilon)),
        Geometry::Polygon(g) => Geometry::Polygon(g.simplifyvw_preserve(&epsilon)),
        Geometry::MultiLineString(g) => Geometry::MultiLineString(g.simplifyvw_preserve(&epsilon)),
        Geometry::MultiPolygon(g) => Geometry::MultiPolygon(g.simplifyvw_preserve(&epsilon)),
        _ => geom,
    }
}
