extern crate geojson;
extern crate serde_json;
extern crate os_type;

use std::process::Command;
use std::io;
use geojson::GeoJson;
use geoq::reader::Reader;
use geoq::entity;
use geoq::error::Error;
use percent_encoding::{utf8_percent_encode, DEFAULT_ENCODE_SET};

const GEOJSON_IO_URL_LIMIT: usize = 27000;

pub fn run() -> Result<(), Error> {
    let stdin = io::stdin();
    let mut stdin_reader = stdin.lock();
    let reader = Reader::new(&mut stdin_reader);
    let features = reader
        .flat_map(|i| entity::from_input(i))
        .map(|e| e.geojson_feature());

    let fc = geojson::FeatureCollection {
        bbox: None,
        features: features.collect(),
        foreign_members: None,
    };
    let fc_json = GeoJson::from(fc).to_string();

    if fc_json.len() > GEOJSON_IO_URL_LIMIT {
        eprintln!("Input exceeds geojson.io 27k character upload limit.");
        Err(Error::InputTooLarge)
    } else {
        let encoded = utf8_percent_encode(&fc_json, DEFAULT_ENCODE_SET);
        let url = format!("http://geojson.io#data=data:application/json,{}", encoded);

        // TODO: something for windows here?
        let open_command = match os_type::current_platform().os_type {
            os_type::OSType::OSX => "open",
            _ => "xdg-open"
        };

        Command::new(open_command)
            .arg(url)
            .status()
            .expect("Failed to open geojson.io");

        Ok(())
    }
}
