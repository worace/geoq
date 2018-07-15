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

const GEOJSON_IO_URL_LIMIT: usize = 27000;
static GEOJSON_IO_HTML_P1: &'static [u8] = include_bytes!("../../../resources/geojsonio_p1.html");
static GEOJSON_IO_HTML_P2: &'static [u8] = include_bytes!("../../../resources/geojsonio_p2.html");

fn timestamp() -> u64 {
    let start = SystemTime::now();
    start.duration_since(UNIX_EPOCH).expect("Time went backwards").as_secs()
}

fn open(media: String) -> () {
    let open_command = match os_type::current_platform().os_type {
        os_type::OSType::OSX => "open",
        _ => "xdg-open"
    };

    Command::new(open_command)
        .arg(media.clone())
        .status()
        .expect(&format!("Failed to open media: {}", media));
}

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

    if fc_json.len() < GEOJSON_IO_URL_LIMIT {
        let encoded = utf8_percent_encode(&fc_json, DEFAULT_ENCODE_SET);
        let url = format!("http://geojson.io#data=data:application/json,{}", encoded);
        open(url);

        Ok(())
    } else {
        let tmpfile = format!("/tmp/geoq_map_{}.html", timestamp());
        let mut file = try!(File::create(tmpfile.clone()));
        eprintln!("Opening geojson.io map file: {}", tmpfile);

        try!(file.write_all(GEOJSON_IO_HTML_P1));
        try!(file.write_all(fc_json.as_bytes()));
        try!(file.write_all(GEOJSON_IO_HTML_P2));
        open(tmpfile);

        Ok(())
    }
}
