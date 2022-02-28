#[derive(Debug, Clone, PartialEq)]
pub struct BBox {
    pub min_x: f64,
    pub min_y: f64,
    pub max_x: f64,
    pub max_y: f64,
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

    fn expand_geom(&mut self, geom: &geojson::Value) {
        match geom {
            geojson::Value::Point(coords) => self.expand_vec(&coords),
            geojson::Value::MultiPoint(coords) => self.expand_vec_vec(&coords),
            geojson::Value::LineString(coords) => self.expand_vec_vec(&coords),
            geojson::Value::MultiLineString(coords) => self.expand_vec_vec_vec(&coords),
            geojson::Value::Polygon(coords) => self.expand_vec_vec_vec(&coords),
            geojson::Value::MultiPolygon(coords) => self.expand_vec_vec_vec_vec(&coords),
            geojson::Value::GeometryCollection(geoms) => {
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
        let mut bb = BBox::empty();
        bb.expand_feature(feat);
        bb
    }

    pub fn to_vec(&self) -> Vec<f64> {
        vec![self.min_x, self.min_y, self.max_x, self.max_y]
    }

    pub fn center(&self) -> (f64, f64) {
        (
            (self.min_x + self.max_x) / 2.0,
            (self.min_y + self.max_y) / 2.0,
        )
    }

    pub fn width(&self) -> f64 {
        self.max_x - self.min_x
    }

    pub fn height(&self) -> f64 {
        self.max_y - self.min_y
    }
}
