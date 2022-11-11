#![feature(try_blocks)]
mod geoq;
use geoq::commands;
use geoq::error::Error;
use geoq::text;

use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};
use std::process;

fn run(matches: ArgMatches) -> Result<(), Error> {
    match matches.subcommand() {
        ("wkt", Some(_)) => commands::wkt::run(),
        ("read", Some(_)) => commands::read::run(),
        ("gj", Some(m)) => commands::geojson_cmd::run(m),
        ("gh", Some(m)) => commands::geohash::run(m),
        ("map", Some(_)) => commands::map::run(),
        ("snip", Some(_)) => commands::snip::run(),
        ("filter", Some(m)) => commands::filter::run(m),
        ("json", Some(m)) => commands::json::run(m),
        ("centroid", Some(_)) => commands::centroid::run(),
        ("whereami", Some(_)) => commands::whereami::run(),
        ("simplify", Some(m)) => commands::simplify::run(m),
        ("measure", Some(m)) => commands::measure::run(m),
        ("bbox", Some(m)) => commands::bbox::run(m),
        ("shp", Some(m)) => commands::shp::run(m),
        ("fgb", Some(m)) => commands::fgb::run(m),
        ("h3", Some(m)) => commands::h3::run(m),
        _ => Err(Error::UnknownCommand),
    }
}

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

fn main() {
    let geojson = SubCommand::with_name("gj")
        .about("Output features as GeoJSON")
        .subcommand(SubCommand::with_name("geom").about("Output entity as a GeoJSON geometry"))
        .subcommand(SubCommand::with_name("f").about("Output entity as a GeoJSON Feature"))
        .subcommand(
            SubCommand::with_name("fc")
                .about("Collect all given entities into a GeoJSON Feature Collection"),
        );

    let geohash = SubCommand::with_name("gh")
        .about("Work with geohashes")
        .subcommand(
            SubCommand::with_name("point")
                .about("Output base 32 Geohash for a given Lat,Lon")
                .arg(
                    Arg::with_name("level")
                        .help("Characters of geohash precision")
                        .required(true)
                        .index(1),
                ),
        )
        .subcommand(
            SubCommand::with_name("covering")
                .about("Output the set of geohashes at the given level which covers the given entity.")
                .arg(
                    Arg::with_name("level")
                        .help("Characters of geohash precision")
                        .required(true)
                        .index(1),
                ).arg(Arg::with_name("original")
                      .long("original")
                      .short("o")
                      .help("Also print the query entity in the output.\nUseful for mapping a geometry along with its covering Geohashes.")),
        )
        .subcommand(SubCommand::with_name("children").about("Get children for the given geohash"))
        .subcommand(SubCommand::with_name("roots").about("List the Base32 Geohash root characters"))
        .subcommand(SubCommand::with_name("encode-long").about("Convert a 64 bit geohash from Base 10 numeric representation to Base 32."))
        .subcommand(SubCommand::with_name("neighbors")
                    .about("Get neighbors of the given Geohash")
                    .arg(Arg::with_name("exclude")
                         .long("exclude")
                         .short("e")
                         .help("Exclude the given geohash from its neighbors.\nBy default it will be included in the output,\ngiving a 3x3 grid centered on the provided geohash.")));

    let filter = SubCommand::with_name("filter")
        .about("Select features based on geospatial predicates")
        .after_help(text::FILTER_AFTER_HELP)
        .arg(Arg::with_name("query-file")
             .help("Input file for reading query feature(s).")
             .takes_value(true)
             .global(true)
             .long("query-file")
             .short("q"))
        .arg(Arg::with_name("negate")
             .help("Negate the filter, so intersects becomes 'not intersects', etc.")
             .global(true)
             .long("negate")
             .short("n"))
        .subcommand(
            SubCommand::with_name("intersects")
                .about("Output only entities (from STDIN) which intersect a QUERY entity (as command-line ARG)")
                .arg(Arg::with_name("query")
                     .help("Entity to check intersections.\nMust be Lat/Lon, Geohash, WKT, or GeoJSON.")
                     .index(1))
        )
        .subcommand(
            SubCommand::with_name("contains")
                .about("Output only entities (from STDIN) which fall within a QUERY entity (as command-line ARG)")
                .arg(
                    Arg::with_name("query")
                        .help("Entity to check intersections.\nMust be Geohash, WKT, or GeoJSON.\nMust be a POLYGON or MULTIPOLYGON.")
                        .index(1)
                )
        )
        .subcommand(
            SubCommand::with_name("dwithin")
                .about("Output only points (from STDIN) which fall within a QUERY entity (as command-line ARG)")
                .after_help(text::FILTER_DWITHIN_AFTER_HELP)
                .arg(
                    Arg::with_name("query")
                        .help("Feature(s) to check intersections.\nMust be Geohash, WKT, or GeoJSON.")
                        .index(1)
                )
                .arg(
                    Arg::with_name("radius")
                        .help("Radius in meters")
                        .takes_value(true)
                        .required(true)
                        .long("radius")
                        .short("r")
                )
        );

    let json = SubCommand::with_name("json")
        .about("Best-guess conversions from geo-oriented JSON to GeoJSON")
        .subcommand(
            SubCommand::with_name("munge")
                .about("Attempt to convert arbitrary JSON to a GeoJSON Feature.")
                .after_help(text::JSON_MUNGE_AFTER_HELP),
        );

    let read = SubCommand::with_name("read")
        .about("Information about reading inputs with geoq")
        .after_help(text::READ_AFTER_HELP);

    let centroid = SubCommand::with_name("centroid")
        .about(text::CENTROID_ABOUT)
        .after_help(text::CENTROID_AFTER_HELP);

    let whereami = SubCommand::with_name("whereami")
        .about(text::WHEREAMI_ABOUT)
        .after_help(text::WHEREAMI_AFTER_HELP);

    let measure = SubCommand::with_name("measure")
        .about(text::MEASURE_ABOUT)
        .subcommand(
            SubCommand::with_name("distance")
                .about(text::DISTANCE_ABOUT)
                .after_help(text::DISTANCE_AFTER_HELP)
                .arg(
                    Arg::with_name("query")
                        .help(text::DISTANCE_QUERY_ARG_HELP)
                        .required(true)
                        .index(1),
                ),
        )
        .subcommand(
            SubCommand::with_name("coord-count")
                .about(text::MEASURE_COORDS_ABOUT)
                .arg(
                    Arg::with_name("geojson")
                        .long("geojson")
                        .required(false)
                        .takes_value(false)
                        .help(text::MEASURE_COORDS_GEOJSON_ARG_HELP),
                ),
        );

    let simplify = SubCommand::with_name("simplify")
        .about(text::SIMPLIFY_ABOUT)
        .after_help(text::SIMPLIFY_AFTER_HELP)
        .arg(
            Arg::with_name("epsilon")
                .help(text::SIMPLIFY_EPSILON_ARG_HELP)
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("to_coord_count")
                .long("to-coord-count")
                .required(false)
                .takes_value(true)
                .help(text::SIMPLIFY_TO_COORD_COUNT_ARG_HELP),
        );

    let bbox = SubCommand::with_name("bbox")
        .about("Generate bounding boxes for geometries")
        .arg(Arg::with_name("embed").long("embed").short("e").help(
            "Print inputs as GeoJSON features and include the bbox in the GeoJSON bbox field",
        ))
        .arg(
            Arg::with_name("all")
                .long("all")
                .short("a")
                .help("Give a single bbox for all input geometries rather than 1 bbox per input"),
        )
        .after_help(text::BBOX_AFTER_HELP);

    let shp = SubCommand::with_name("shp")
        .about("Read a shapefile and convert to GeoJSON")
        .arg(
            Arg::with_name("path")
                .help("output file, e.g. data.fgb")
                .required(true)
                .index(1),
        );

    let fgb = SubCommand::with_name("fgb")
        .about("Reading and Writing FlatGeoBuf")
        .subcommand(
            SubCommand::with_name("write")
                .about("Write GeoJSON data to a binary flatgeobuf file")
                .arg(
                    Arg::with_name("path")
                        .help("output path")
                        .required(true)
                        .index(1),
                ),
        )
        .subcommand(
            SubCommand::with_name("read")
                .about("Read a binary flatgeobuf file to GeoJSON")
                .arg(
                    Arg::with_name("path")
                        .help("input path to .fgb file")
                        .required(true)
                        .index(1),
                )
                .arg(
                    Arg::with_name("bbox")
                        .allow_hyphen_values(true)
                        .long("bbox")
                        .required(false)
                        .takes_value(true)
                        .help("Comma-separated bounding box: minLon,minLat,maxLon,maxLat"),
                ),
        );

    let h3 = SubCommand::with_name("h3")
        .about("Work with H3")
        .subcommand(
            SubCommand::with_name("point")
                .about("Output hexadecimal encoded Cell ID for a given Lat,Lon at requested resolution")
                .arg(
                    Arg::with_name("resolution")
                        .help("H3 cell resolution (0-15)")
                        .required(true)
                        .index(1),
                ),
        )
        .subcommand(
            SubCommand::with_name("hierarchy")
                .about("Output all hexadecimal encoded Cells for a given Lat,Lon, from res 0 to 15")
        )
        .subcommand(
            SubCommand::with_name("to-str")
                .about("Convert an h3 numeric index (64-bit integer representation) to its hexadecimal string representation")
        ).subcommand(
            SubCommand::with_name("from-str")
                .about("Convert an h3 string index (15-character hexadecimal representation) to its 64-bit integer numeric representation")
        ).subcommand(
            SubCommand::with_name("children")
                .about("Get children for given cells at given resolution. If no resolution is given, the cell's resolution + 1 is used.")
                .arg(
                    Arg::with_name("resolution")
                        .help("H3 cell resolution (0-15)")
                        .index(1),
                )
        ).subcommand(
            SubCommand::with_name("parent")
                .about("Get parent (or ancestor) for given cells at given resolution. If no resolution is given, the cell's resolution - 1 (immediate parent) is used.")
                .arg(
                    Arg::with_name("resolution")
                        .help("H3 cell resolution (0-15)")
                        .index(1),
                )
        ).subcommand(SubCommand::with_name("resolution").about("Get resolution for an H3 cell"))
        .subcommand(
            SubCommand::with_name("grid-disk")
                .about("Get disk of given radius around given cells. Default radius is 1.")
                .arg(
                    Arg::with_name("radius")
                        .help("https://h3geo.org/docs/api/traversal/#griddisk")
                        .index(1),
                )
        ).subcommand(
            SubCommand::with_name("polyfill")
                .about("Generate set of H3 cells covering a polygon or multipolygon.")
                .help(text::H3_POLYFILL_HELP)
                .arg(Arg::with_name("min-res").long("min-res").help(
                    "Minimum (coarsest) resolution for cell set (default: 0)",
                ).takes_value(true))
                .arg(Arg::with_name("max-res").long("max-res").help(
                    "Maximum (finest) resolution for cell set (default: 15)",
                ).takes_value(true))
                .arg(Arg::with_name("original")
                     .long("original")
                     .short("o")
                     .help("Also print the query entity in the output.\nUseful for mapping a geometry along with its covering H3 Cells."))
        ).subcommand(
            SubCommand::with_name("covering")
                .about("Generate set of H3 cells covering a geometry.")
                .arg(
                    Arg::with_name("resolution")
                        .help("H3 cell resolution (0-15)")
                        .index(1),
                )
                .arg(Arg::with_name("original")
                     .long("original")
                     .short("o")
                     .help("Also print the query entity in the output.\nUseful for mapping a geometry along with its covering H3 Cells."))
        ).subcommand(
            SubCommand::with_name("polyfill-h3")
                .about("Generate set of H3 cells covering a polygon or multipolygon.")
                .help(text::H3_POLYFILL_HELP)
                .arg(Arg::with_name("min-res").long("min-res").help(
                    "Minimum (coarsest) resolution for cell set (default: 0)",
                ).takes_value(true))
                .arg(Arg::with_name("max-res").long("max-res").help(
                    "Maximum (finest) resolution for cell set (default: 15)",
                ).takes_value(true))
                .arg(Arg::with_name("original")
                     .long("original")
                     .short("o")
                     .help("Also print the query entity in the output.\nUseful for mapping a geometry along with its covering H3 Cells."))
        );

    let matches = App::new("geoq")
        .version(VERSION)
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .about("geoq - GeoSpatial utility belt")
        .after_help(text::MAIN_AFTER_HELP)
        .subcommand(SubCommand::with_name("wkt").about("Output features as Well-Known Text"))
        .subcommand(SubCommand::with_name("map").about("View features on a map using geojson.io"))
        .subcommand(read)
        .subcommand(geohash)
        .subcommand(geojson)
        .subcommand(json)
        .subcommand(filter)
        .subcommand(centroid)
        .subcommand(whereami)
        .subcommand(measure)
        .subcommand(simplify)
        .subcommand(bbox)
        .subcommand(shp)
        .subcommand(fgb)
        .subcommand(h3)
        .get_matches();

    if let Err(e) = run(matches) {
        eprintln!("geoq exited with error: {:?}", e);
        process::exit(1);
    }
}
