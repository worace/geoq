extern crate geojson;
extern crate serde_json;
extern crate os_type;

use std::process::Command;
use std::io;
use geojson::GeoJson;
use geoq::reader::Reader;
use geoq::error::Error;
use percent_encoding::{utf8_percent_encode, DEFAULT_ENCODE_SET};
use std::time::{SystemTime, UNIX_EPOCH};
use std::fs::File;
use std::io::prelude::*;
use geoq::browser_open;

const SNIP_LIMIT: usize = 10000000; // 10mb

pub fn run() -> Result<(), Error> {
    let stdin = io::stdin();
    let mut stdin_reader = stdin.lock();
    let reader = Reader::new(&mut stdin_reader);

    let mut features: Vec<geojson::Feature> = Vec::new();

    for e_res in reader {
        match e_res {
            Ok(entity) => features.push(entity.geojson_feature()),
            Err(e) => return Err(e)
        }
    }

    let fc = geojson::FeatureCollection {
        bbox: None,
        features: features,
        foreign_members: None,
    };
    let fc_json = GeoJson::from(fc).to_string();

    if fc_json.len() < SNIP_LIMIT {
        let client = reqwest::Client::builder()
            .redirect(reqwest::RedirectPolicy::none())
            .build()?;
        let resp = client.post("https://contour.app/scratchpad")
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .body(fc_json)
            .send()?;
        if let Some(loc) = resp.headers().get(reqwest::header::LOCATION) {
            let url = loc.to_str().unwrap();
            eprintln!("Opening Contour Scratchpad:\n{}", url);
            browser_open::open(url.to_string());
        }
        Ok(())
    } else {
        Err(Error::InputTooLarge)
    }
}
