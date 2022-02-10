use std::convert::TryInto;

use geo::coords_iter;
use geojson::{Feature, Value};

#[derive(Debug, Clone, PartialEq)]
pub struct BBox {
    pub min_x: f64,
    pub min_y: f64,
    pub max_x: f64,
    pub max_y: f64,
}

#[derive(Debug)]
pub struct BoundedFeature {
    pub feature: Feature,
    pub bbox: BBox,
}

#[derive(Debug, Clone)]
pub struct IndexNode {
    pub offset: usize,
    pub bbox: BBox,
}

fn f64_from_bytes(bytes: &[u8]) -> Result<f64, &str> {
    let arr: [u8; 8] = bytes.try_into().map_err(|_| "Expected 8 bytes for f64.")?;

    Ok(f64::from_le_bytes(arr))
}
fn u64_from_bytes(bytes: &[u8]) -> Result<u64, &str> {
    let arr: [u8; 8] = bytes.try_into().map_err(|_| "Expected 8 bytes for u64.")?;

    Ok(u64::from_le_bytes(arr))
}

impl IndexNode {
    pub fn from_bytes(bytes: &[u8]) -> Result<IndexNode, &str> {
        if bytes.len() < 40 {
            return Err("Not enough bytes for IndexNode");
        }
        let min_x = f64_from_bytes(&bytes[0..8])?;
        let min_y = f64_from_bytes(&bytes[8..16])?;
        let max_x = f64_from_bytes(&bytes[16..24])?;
        let max_y = f64_from_bytes(&bytes[24..32])?;
        let offset = u64_from_bytes(&bytes[32..40])? as usize;
        let bbox = BBox {
            min_x,
            min_y,
            max_x,
            max_y,
        };

        Ok(IndexNode { bbox, offset })
    }
}

impl BBox {
    pub fn new(x: f64, y: f64) -> BBox {
        BBox {
            min_x: x,
            min_y: y,
            max_x: x,
            max_y: y,
        }
    }

    pub fn empty() -> BBox {
        BBox {
            min_x: f64::INFINITY,
            min_y: f64::INFINITY,
            max_x: f64::NEG_INFINITY,
            max_y: f64::NEG_INFINITY,
        }
    }

    pub fn expand(&mut self, other: &BBox) {
        if other.min_x < self.min_x {
            self.min_x = other.min_x;
        }
        if other.min_y < self.min_y {
            self.min_y = other.min_y;
        }
        if other.max_x > self.max_x {
            self.max_x = other.max_x;
        }
        if other.max_y > self.max_y {
            self.max_y = other.max_y;
        }
    }

    fn expand_xy(&mut self, x: f64, y: f64) {
        if x < self.min_x {
            self.min_x = x;
        }
        if y < self.min_y {
            self.min_y = y;
        }
        if x > self.max_x {
            self.max_x = x;
        }
        if y > self.max_y {
            self.max_y = y;
        }
    }

    fn expand_vec(&mut self, coords: &Vec<f64>) {
        self.expand_xy(coords[0], coords[1]);
    }

    fn expand_vec_vec(&mut self, coords: &Vec<Vec<f64>>) {
        for coord in coords {
            self.expand_vec(coord);
        }
    }

    fn expand_vec_vec_vec(&mut self, rings: &Vec<Vec<Vec<f64>>>) {
        for ring in rings {
            self.expand_vec_vec(ring);
        }
    }

    fn expand_vec_vec_vec_vec(&mut self, polys: &Vec<Vec<Vec<Vec<f64>>>>) {
        for poly in polys {
            self.expand_vec_vec_vec(poly);
        }
    }

    fn expand_geom(&mut self, geom: &Value) {
        match geom {
            Value::Point(coords) => self.expand_vec(&coords),
            Value::MultiPoint(coords) => self.expand_vec_vec(&coords),
            Value::LineString(coords) => self.expand_vec_vec(&coords),
            Value::MultiLineString(coords) => self.expand_vec_vec_vec(&coords),
            Value::Polygon(coords) => self.expand_vec_vec_vec(&coords),
            Value::MultiPolygon(coords) => self.expand_vec_vec_vec_vec(&coords),
            Value::GeometryCollection(geoms) => {
                for geom in geoms {
                    self.expand_geom(&geom.value)
                }
            }
        }
    }

    pub fn expand_feature(&mut self, feat: &geojson::Feature) {
        if feat.geometry.is_none() {
            return;
        }

        let g = &feat.geometry.as_ref().unwrap().value;
        self.expand_geom(g);
    }

    pub fn for_feature(feat: &geojson::Feature) -> BBox {
        let (x, y) = feat_coord(feat);
        let mut bb = BBox::new(x, y);
        bb.expand_feature(feat);
        bb
    }

    pub fn to_vec(&self) -> Vec<f64> {
        vec![self.min_x, self.min_y, self.max_x, self.max_y]
    }

    fn center(&self) -> (f64, f64) {
        (
            (self.min_x + self.max_x) / 2.0,
            (self.min_y + self.max_y) / 2.0,
        )
    }

    fn width(&self) -> f64 {
        self.max_x - self.min_x
    }

    fn height(&self) -> f64 {
        self.max_y - self.min_y
    }

    fn hilbert_bbox(&self, extent: &BBox) -> u32 {
        // calculate bbox center and scale to hilbert_max
        let (mid_x, mid_y) = self.center();
        let x = (HILBERT_MAX * mid_x / extent.width()).floor() as u32;
        let y = (HILBERT_MAX * mid_y / extent.height()).floor() as u32;
        hilbert(x, y)
    }
}

fn feat_coord(f: &geojson::Feature) -> (f64, f64) {
    f.geometry.as_ref().map(|geom| coord(&geom.value)).unwrap()
}
fn coord(geom: &Value) -> (f64, f64) {
    let o = match geom {
        Value::Point(coords) => Some((coords[0], coords[1])),
        Value::MultiPoint(coords) => coords.first().map(|c| (c[0], c[1])),
        Value::LineString(coords) => coords.first().map(|c| (c[0], c[1])),
        Value::Polygon(rings) => rings.first().and_then(|r| r.first().map(|c| (c[0], c[1]))),
        Value::MultiLineString(lines) => lines
            .first()
            .and_then(|line| line.first().map(|c| (c[0], c[1]))),
        Value::MultiPolygon(polys) => polys
            .first()
            .and_then(|rings| rings.first().and_then(|r| r.first().map(|c| (c[0], c[1])))),
        Value::GeometryCollection(geoms) => geoms.first().map(|geom| coord(&geom.value)),
    };
    o.unwrap()
}

const HILBERT_MAX: f64 = ((1 << 16u32) - 1) as f64;

pub fn sort_with_extent(features: Vec<geojson::Feature>) -> (Vec<BoundedFeature>, BBox) {
    let (start_x, start_y) = features.first().map(|f| feat_coord(f)).unwrap();
    let mut extent = BBox::new(start_x, start_y);
    let mut bounded_feats: Vec<BoundedFeature> = features
        .into_iter()
        .map(|f| {
            let bb = BBox::for_feature(&f);
            extent.expand(&bb);
            BoundedFeature {
                feature: f,
                bbox: bb,
            }
        })
        .collect();
    bounded_feats.sort_by(|a, b| {
        a.bbox
            .hilbert_bbox(&extent)
            .partial_cmp(&b.bbox.hilbert_bbox(&extent))
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    (bounded_feats, extent)
}

// Based on public domain code at https://github.com/rawrunprotected/hilbert_curves
fn hilbert(x: u32, y: u32) -> u32 {
    let mut a = x ^ y;
    let mut b = 0xFFFF ^ a;
    let mut c = 0xFFFF ^ (x | y);
    let mut d = x & (y ^ 0xFFFF);

    let mut aa = a | (b >> 1);
    let mut bb = (a >> 1) ^ a;
    let mut cc = ((c >> 1) ^ (b & (d >> 1))) ^ c;
    let mut dd = ((a & (c >> 1)) ^ (d >> 1)) ^ d;

    a = aa;
    b = bb;
    c = cc;
    d = dd;
    aa = (a & (a >> 2)) ^ (b & (b >> 2));
    bb = (a & (b >> 2)) ^ (b & ((a ^ b) >> 2));
    cc ^= (a & (c >> 2)) ^ (b & (d >> 2));
    dd ^= (b & (c >> 2)) ^ ((a ^ b) & (d >> 2));

    a = aa;
    b = bb;
    c = cc;
    d = dd;
    aa = (a & (a >> 4)) ^ (b & (b >> 4));
    bb = (a & (b >> 4)) ^ (b & ((a ^ b) >> 4));
    cc ^= (a & (c >> 4)) ^ (b & (d >> 4));
    dd ^= (b & (c >> 4)) ^ ((a ^ b) & (d >> 4));

    a = aa;
    b = bb;
    c = cc;
    d = dd;
    cc ^= (a & (c >> 8)) ^ (b & (d >> 8));
    dd ^= (b & (c >> 8)) ^ ((a ^ b) & (d >> 8));

    a = cc ^ (cc >> 1);
    b = dd ^ (dd >> 1);

    let mut i0 = x ^ y;
    let mut i1 = b | (0xFFFF ^ (i0 | a));

    i0 = (i0 | (i0 << 8)) & 0x00FF00FF;
    i0 = (i0 | (i0 << 4)) & 0x0F0F0F0F;
    i0 = (i0 | (i0 << 2)) & 0x33333333;
    i0 = (i0 | (i0 << 1)) & 0x55555555;

    i1 = (i1 | (i1 << 8)) & 0x00FF00FF;
    i1 = (i1 | (i1 << 4)) & 0x0F0F0F0F;
    i1 = (i1 | (i1 << 2)) & 0x33333333;
    i1 = (i1 | (i1 << 1)) & 0x55555555;

    let value = (i1 << 1) | i0;

    value
}
