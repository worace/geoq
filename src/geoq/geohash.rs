extern crate geohash;
extern crate geo_types;
extern crate geo;

use geo_types::{Geometry, Polygon, LineString, Point, Coordinate};
use geoq::intersection;
use geoq::contains;

pub const BASE_32: [char; 32] = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'b', 'c', 'd', 'e', 'f', 'g',
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

    let neighbs = geohash::neighbors(gh).unwrap();
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

pub fn bbox(gh: &str) -> Option<Polygon<f64>> {
    match geohash::decode_bbox(gh) {
        Ok(rect) => {
            let bl = rect.min;
            let tr = rect.max;
            let outer = LineString(vec![
                Coordinate::from((bl.x, bl.y)),
                Coordinate::from((tr.x, bl.y)),
                Coordinate::from((tr.x, tr.y)),
                Coordinate::from((bl.x, tr.y)),
                Coordinate::from((bl.x, bl.y)),
            ]);
            Some(Polygon::new(outer, Vec::new()))
        }
        _ => None
    }
}

pub fn covering(geom: &Geometry<f64>, level: usize) -> Vec<String> {
    let mut ghs: Vec<String> = vec![];
    let mut queue: Vec<String> = vec!["".to_string()];
    while !queue.is_empty() {
        let gh = queue.pop().unwrap();
        match bbox(&gh) {
            Some(poly) => {
                if contains::contains(&poly, &geom) || intersection::poly_intersects(&poly, &geom) {
                    if gh.len() < level {
                        queue.extend(children(&gh));
                    } else {
                        ghs.push(gh);
                    }
                }
            }
            None => ()
        }
    }
    ghs
}
