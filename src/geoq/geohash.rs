use crate::geoq::contains;
use geo_types::{Coordinate, Geometry, LineString, Polygon};
use std::str;

pub const BASE_32: [char; 32] = [
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'j', 'k',
    'm', 'n', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
];

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
    if gh == "" {
        let min = Coordinate::<f64>::from((-180.0, -90.0));
        let max = Coordinate::<f64>::from((180.0, 90.0));
        return Some(geo_types::Rect::new(min, max).to_polygon());
    }
    match geohash::decode_bbox(gh) {
        Ok(rect) => {
            let bl = rect.min();
            let tr = rect.max();
            let outer = LineString(vec![
                Coordinate::from((bl.x, bl.y)),
                Coordinate::from((tr.x, bl.y)),
                Coordinate::from((tr.x, tr.y)),
                Coordinate::from((bl.x, tr.y)),
                Coordinate::from((bl.x, bl.y)),
            ]);
            Some(Polygon::new(outer, Vec::new()))
        }
        _ => None,
    }
}

pub fn covering(geom: &Geometry<f64>, level: usize) -> Vec<String> {
    use geo::algorithm::intersects::Intersects;
    let mut ghs: Vec<String> = vec![];
    let mut queue: Vec<String> = vec!["".to_string()];
    while !queue.is_empty() {
        let gh = queue.pop().unwrap();
        match bbox(&gh) {
            Some(poly) => {
                if contains::contains(&poly, &geom) || poly.intersects(geom) {
                    if gh.len() < level {
                        queue.extend(children(&gh));
                    } else {
                        ghs.push(gh);
                    }
                }
            }
            None => (),
        }
    }
    ghs
}

// Copied from https://github.com/tidwall/geohash-rs/blob/master/src/base32.rs
// TODO: Move remaining geohash usage to this crate, but for now I just
// wanted the encoding stuff without importing 2 separate geohash libs
const BASE32_ENCODING: [u8; 32] = [
    b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9', b'b', b'c', b'd', b'e', b'f', b'g',
    b'h', b'j', b'k', b'm', b'n', b'p', b'q', b'r', b's', b't', b'u', b'v', b'w', b'x', b'y', b'z',
];

pub fn encode_long(mut x: u64) -> String {
    let mut bytes = [0u8; 12];
    for i in 0..12 {
        bytes[11 - i] = BASE32_ENCODING[x as usize & 0x1f];
        x >>= 5;
    }
    str::from_utf8(&bytes).unwrap().to_string()
}
