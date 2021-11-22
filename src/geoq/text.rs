pub const JSON_MUNGE_AFTER_HELP: &str = r#"
Create a GeoJSON Feature from arbitrary JSON by checking some common
patterns used for embedding geo data in JSON objects.

The checks include:

* latitude and longitude under the lat, latitude, lon, longitude, or lng keys
* WKT strings under the geometry or wkt keys
* GeoJSON geometries as strings under the geometry or geojson keys
* GeoJSON geometries as objects under the geometry or geojson keys

The original JSON object, minus the matched geometry keys, will be embedded in the GeoJSON 'properties' key.

Example:

$ echo '{"latitude":12.0,"longitude":34.0,"key":"val"}' | geoq json munge
{"geometry":{"coordinates":[34.0,12.0],"type":"Point"}, "properties":{"key":"val"},"type":"Feature"}
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

pub const WHEREAMI_ABOUT: &str = "Get IP-based current lat/lon.";
pub const WHEREAMI_AFTER_HELP: &str = r"
Get IP-based current lat/lon.

Location is reported using a free service at ip-api.com.

Rate-limited to 150 requests per minute from a given IP.

Output is given as a GeoJSON Point.
";

pub const MEASURE_ABOUT: &str = "Get spatial measurements of or between features.";
pub const DISTANCE_ABOUT: &str =
    "Output distance between features (from STDIN) and a QUERY feature (as command-line ARG)";
pub const DISTANCE_QUERY_ARG_HELP: &str = r"
Feature to measure from.

Query Feature must be a POINT, and can be provided as
Lat/Lon, WKT, or GeoJSON.
";

pub const DISTANCE_AFTER_HELP: &str = r"
Gives the distance in meters between each input Feature and the given QUERY Feature.

Output is given in the format:

<Distance><TAB><Feature>

e.g.

12.3	LINESTRING (30 10, 10 30, 40 40)

This enables the output to be processed with other unix commands
like 'sort', 'cut', etc.

Distances will be given between the QUERY point and the closest
point on each input Feature.

Distance for Features that cover the QUERY point (like a Polygon containing the point) will be 0.
";

pub const FILTER_AFTER_HELP: &str = r"
Select features based on geospatial predicates

'filter' commands require 1 or more query geometries to
be used for checking predicates.

Queries can be provided as either a file input using the
--query-file argument, or as a positional argument on the command line.

The input format for queries is the same as normal geoq inputs.
(See 'geoq help read' for more info)

For example, to check inputs (via STDIN) against queries from a file:

geoq filter intersects --query-file /path/to/inputs

Or, to check against inputs from the command line:

geoq filter intersects 9q5

geoq filter contains 'Polygon ((30 10, 10 30, 40 40, 30 10))'
";

pub const SIMPLIFY_ABOUT: &str = "Simplify geometries.";

pub const SIMPLIFY_AFTER_HELP: &str = r"Reads features from STDIN.

Uses the Visvalingham-Whyatt topology-preserving simplification algorithm.
(https://www.jasondavies.com/simplify/)

Only (Multi-)LineStrings and (Multi-)Polygons will be affected.

Takes Epsilon as a command-line parameter

If the optional --to-size arg is given, geoq will iteratively simplify each
given geometry until it is under this target number of vertices, starting
from the provided epsilon and doubling on each attempt.

The iterative simplification will stop after 20 attempts, so it's still
good to check the coord-count of each geometry afterward to determine
if any rows were unable to be simplified under the desired threshold.
";

pub const SIMPLIFY_EPSILON_ARG_HELP: &str = r"
Simplification epsilon as float, e.g 0.001.
";

pub const SIMPLIFY_TO_COORD_COUNT_ARG_HELP: &str = r"
Target number of coords to simplify to.
";

pub const MEASURE_COORDS_ABOUT: &str =
    "Count number of total coordinates/vertices in each feature. Use --geojson to get results embedded in a GeoJSON Feature as a property.";
pub const MEASURE_COORDS_GEOJSON_ARG_HELP: &str =
    "Give result as an embedded property in a GeoJSON feature, rather than as a single number.";

pub const BBOX_AFTER_HELP: &str = "Generate bbox for a geometry";
