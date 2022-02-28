use std::convert::TryInto;

use geo::coords_iter;
use geojson::{Feature, Value};

use super::bbox::BBox;

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
        let hilbert_a = hilbert_bbox(&a.bbox, &extent);
        let hilbert_b = hilbert_bbox(&b.bbox, &extent);

        hilbert_a
            .partial_cmp(&hilbert_b)
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

pub fn hilbert_bbox(bbox: &BBox, extent: &BBox) -> u32 {
    // calculate bbox center and scale to hilbert_max
    let (mid_x, mid_y) = bbox.center();
    let x = (HILBERT_MAX * (mid_x - extent.min_x) / extent.width()).floor() as u32;
    let y = (HILBERT_MAX * (mid_y - extent.min_y) / extent.height()).floor() as u32;
    hilbert(x, y)
}
