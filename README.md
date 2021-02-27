# geoq

[![Build Status](https://travis-ci.org/worace/geoq.svg?branch=master)](https://travis-ci.org/worace/geoq)
[![crates.io](https://img.shields.io/badge/crates.io-v0.0.5-orange.svg)](https://crates.io/crates/geoq)

Geoq is a command-line tool for working with geospatial data.

It combines common GIS serialization formats with utilities for manipulating and visualizing data on a map.

For example:

```sh
# Print the geometry and the set of level 2 geohashes which covers the given geometry
$ echo '{"type":"Polygon","coordinates":[[[-34,38],[-37,32],[-23,33],[-34,38]]]}' | geoq gh covering 2 -o
{"type":"Polygon","coordinates":[[[30,10],[40,40],[20,40],[10,20],[30,10]]]}
eq
en
em
ej

# Feed that output into a map on geojson.io
$ echo '{"type":"Polygon","coordinates":[[[-34,38],[-37,32],[-23,33],[-34,38]]]}' | geoq gh covering 2 -o | geoq map
```

See the [Manual](https://github.com/worace/geoq/blob/master/manual.md) for more examples and available commands.

## Install

Currently installation is done through [crates.io](http://crates.io/), the Rust package repository.

If you have Rust installed, simply:

```
cargo install geoq
```

To [install Rust](https://www.rust-lang.org/en-US/install.html) and the Cargo toolchain:

```
curl https://sh.rustup.rs -sSf | sh
```

You'll also need to add Cargo's `bin` directory to your path:

```
# e.g. in your ~/.bash_profile or other shell profile
export PATH="$HOME/.cargo/bin:$PATH"
```

### Updating an Existing Installation

To pull and install a newer version from crates.io, run:

```
cargo install geoq --force
```

## Supported Input Formats

Geoq will detect the following GIS input formats automatically:

* Comma-separated Lat/Lon: `34.0,-118.0`
* Geohashes (base 32): `9q5`
* WKT: `POINT (-118.0, 34.0)`
* GeoJSON: `{"type": "Point", "coordinates": [-118.0, 34.0]}`

## One Feature Per Line, One Line Per Feature

Geoq processes text inputs on a per-line basis, and it expects inputs not to stretch across multiple lines.

This sometimes causes problems, especially with GeoJSON, because many JSON processing tools like to output pretty-printed JSON in a multi-line format.

One way to fix this problem with pretty-printed GeoJSON is to use the [jq](https://stedolan.github.io/jq/) tool:

```
echo '{
    "type": "Point",
    "coordinates": [30, 10]
}
' | jq -c . | geoq map
```

## Commands

See the built-in command help using `geoq --help` or `geoq <subcommand> --help` for more detailed information on these:

* `gj` - GeoJSON subcommands
  * `f` - Output geometry as GeoJSON feature
  * `geom` - Output geometry as GeoJSON geometry
  * `fc` - Collect all input geometries into a GeoJSON Feature Collection
* `centroid` - Cet the centroid of a geometry
* `filter` - Spatial predicate filtering
  * `intersects` - Select features intersecting a given query geometry
  * `contains` - Select features contained by a given query geometry
* `gh` - Geohash subcommands
  * `children` - Get children of a geohash
  * `covering` - Output geohashes that "cover" a geometry
  * `neighbors` - Get neighbors of a Geohash
  * `point` - Output base 32 Geohash for a given Lat,Lon
* `json` - JSON -> GeoJSON coercion
  * `munge` - Attempt to convert arbitrary JSON to a GeoJSON Feature.
* `map` - Visualization with geojson.io
* `read` - Debugging / format validation
* `wkt` - Output geometries as WKT
* `whereami` - Output IP geolocation-based current lat/lon as GeoJSON
* `measure` - Measurement subcommands
  * `distance` - Measure distances between features


See the [Manual](https://github.com/worace/geoq/blob/master/manual.md) for more examples and available commands.

## Development

### Running Tests

```
cargo test
```

### Building / Releasing

```
cargo publish
git tag release/<VERSION>
git push origin release/<VERSION>
```
