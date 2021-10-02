## Changelog

### 0.0.19

* Added `shp` command to read shapefiles. Currently limited to extracting data and converting to GeoJSON, which can then be passed to other commands for further processing. Very excited to have a pure-rust CLI utility for going from shapefile -> local map preview, or simply extracting shapefile data to more useful formats.

### 0.0.18

* Add `bbox` subcommand for getting bounding box eiterh of individual geometries or of all geometries in a stream
* Add new `--negate` option to the `filter` subcommand. This allows for filtering the inverse of existing predicates, e.g. "does not contain" or "does not intersect"

### 0.0.17

* Add `json munge` subcommand for heuristically extracting GeoJSON from non-GeoJSON JSON objects. `munge` will check for things like "latitude" and "longitude" keys, "wkt" or "geometry" keys containing WKT text, stringified-geojson geometries, etc. Useful for converting output of other assorted scripts into proper GeoJSON.

### 0.0.16

* Fix warnings and update to 2018 rust edition (thanks [stanislav-tkach](https://github.com/worace/geoq/commits?author=stanislav-tkach))
* Add `measure coord-count` command for giving total number of vertices per geometry (useful for rough data size approximations)
* Add `--to-size` arg to `simplify` command for iterative simplification toward target coord count

### 0.0.15

* Add `simplify` subcommand for geometry simplification using Visvalingam–Whyatt algorithm via geo-types.

### 0.0.14

* add `gh encode-long` subcommand for giving u64-encoded geohash values

### 0.0.13

Added an additional `--query-file` argument to the `filter` subcommand for specifying query inputs from a file instead of from the command line.

### 0.0.12

Added `geoq gh roots` command for listing root geohash characters:

```
$ geoq gh roots
0
1
...etc
```

### 0.0.11

Added `geoq measure distance <POINT>` subcommand.

For example:

```
$ echo "9q5\n9qc" | geoq measure distance "POINT(-118.3991 33.9949)"
```

Will output tab-separated distances (in meters) + features:

```
0       9q5
496760.9360151398       9qc
```

Currently the command-line QUERY arg must be a Point (Lat/Lon, WKT, or GeoJSON), but hopefully more geometry types will be supported in the future.
