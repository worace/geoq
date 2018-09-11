pub const JSON_POINT_AFTER_HELP: &str = r#"
Create a GeoJSON Point from arbitrary JSON by searching for common
latitude and longitude property names.

Latitude keys are 'latitude' and 'lat'.
Longitude keys are 'longitude', 'lon', and 'lng'.

The original JSON object will be embedded in the GeoJSON 'properties' key.

Example:

$ echo '{"latitude":12.0,"longitude":34.0,"key":"val"}' | geoq json point
  {"geometry":{"coordinates":[34.0,12.0],"type":"Point"},
   "properties":{"key":"val","latitude":12.0,"longitude":34.0},"type":"Feature"}
"#;

pub const READ_AFTER_HELP: &str = r#"
Geoq reads the following geospatial text formats:

- Comma- or Tab-separated latitude/longitude
- WKT (Well-Known Text)
- GeoJSON
- Geohashes (Base32-encoded)

Values must be submitted **1 per line**. GeoJSON data
in particular is often pretty-printed by other tools, so
take care to ensure your GeoJSON is compacted before feeding
it to Geoq. The 'jq' program is useful for compacting
pretty-printed json: 'jq -cr .'

Geoq will detect the format of values as they are read,
so additional arguments are not needed to specify formats.

Many commands can accept any entity type as input,
although some (for example some of the geohash
subcommands), are restricted to entities of the appropriate
type.

Most entities will be read as 1-per-line. However special
handling is given to GeoJSON 'FeatureCollections'. These
will be "unrolled" by Geoq's processing, and treated as
if their features had been passed individually.

You can also use this command ('geoq read') to test out
input formats and see how geoq reads the values you feed it:

$ printf "LINESTRING (30 10, 10 30)\n9q5\n" | geoq read
  WKT: LINESTRING(30 10,10 30)
  Geohash: 9q5
"#;

pub const MAIN_AFTER_HELP: &str = r#"
Geoq is a streaming command-line program for working with
geospatial data. It combines automatic readers for common
GIS text formats with interfaces to frequent spatial
operations and conversions.

See the listing above for available commands, and use
$ geoq <COMMAND> --help
for additional information on individual commands.

See 'geoq read --help' for more information on supported
input formats.
"#;

pub const CENTROID_ABOUT: &str = "Print centroid of the given geometry";
pub const CENTROID_AFTER_HELP: &str = r"
Output is given as a GeoJSON Point.
";
