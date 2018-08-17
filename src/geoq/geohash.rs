extern crate geohash;
extern crate geo_types;
extern crate geo;

use geo_types::{Geometry, Polygon, LineString, Point};
use geo::algorithm::contains::Contains;
use geoq::intersection;

static BASE_32: [char; 32] = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'b', 'c', 'd', 'e', 'f', 'g',
                              'h', 'j', 'k', 'm', 'n', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z'];

pub fn children(gh: &String) -> Vec<String> {
    BASE_32.iter().map(|c| format!("{}{}", gh, c)).collect()
}

pub fn neighbors(gh: &String, include_self: bool) -> Vec<String> {
    let mut output: Vec<String> = if include_self {
        Vec::with_capacity(9)
    } else {
        Vec::with_capacity(8)
    };

    if include_self {
        output.push(gh.clone());
    }

    let neighbs = geohash::neighbors(gh);
    output.push(neighbs.n);
    output.push(neighbs.ne);
    output.push(neighbs.e);
    output.push(neighbs.se);
    output.push(neighbs.s);
    output.push(neighbs.sw);
    output.push(neighbs.w);
    output.push(neighbs.nw);
    output
}

pub fn bbox(gh: &str) -> Polygon<f64> {
    let (bl, tr) = geohash::decode_bbox(gh);
    let outer = LineString(vec![
        Point::new(bl.x, bl.y),
        Point::new(tr.x, bl.y),
        Point::new(tr.x, tr.y),
        Point::new(bl.x, tr.y),
        Point::new(bl.x, bl.y),
    ]);
    Polygon::new(outer, Vec::new())
}

fn contains(outer: &Polygon<f64>, inner: &Geometry<f64>) -> bool {
    match *inner {
        Geometry::Point(ref g) => outer.contains(g),
        Geometry::LineString(ref g) => outer.contains(g),
        Geometry::Polygon(ref g) => outer.contains(g),
        Geometry::MultiPolygon(ref g) => {
            g.0.iter().all(|poly| outer.contains(poly))
        },
        _ => false
    }
}

pub fn covering(geom: &Geometry<f64>, level: usize) -> Vec<String> {
    let mut ghs: Vec<String> = vec![];
    let mut queue: Vec<String> = vec!["".to_string()];
    while !queue.is_empty() {
        let gh = queue.pop().unwrap();
        let poly = bbox(&gh);
        if contains(&poly, &geom) || intersection::poly_intersects(&poly, &geom) {
            if gh.len() < level {
                queue.extend(children(&gh));
            } else {
                ghs.push(gh);
            }
        }
    }

    ghs
}
