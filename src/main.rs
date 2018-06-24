extern crate clap;
use clap::{App, Arg, SubCommand, ArgMatches};
use std::process;

fn run_wkt(matches: &ArgMatches) -> Result<(), String> {
    println!("RUNNING WKT ***");
    Ok(())
}

fn run(matches: ArgMatches) -> Result<(), String> {
    match matches.subcommand() {
        ("wkt", Some(m)) => run_wkt(&matches),
        _ => Err("Unknown Command".to_string())
    }
}

fn main() {
    let VERSION = "0.1";
    let matches = App::new("geoq")
        .version(VERSION)
        .about("geoq - GeoSpatial utility belt")
        .subcommand(SubCommand::with_name("wkt").about("Output entity as WKT."))
        .get_matches();
    println!("{:?}", matches);
    println!("{:?}", matches.subcommand);
    println!("Hello, world!");

    if let Err(e) = run(matches) {
        println!("Application error: {}", e);
        process::exit(1);
    }
}
