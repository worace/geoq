# geoq

[![Build Status](https://travis-ci.org/worace/geoq.svg?branch=master)](https://travis-ci.org/worace/geoq)
[![crates.io](https://img.shields.io/badge/crates.io-v0.0.1-orange.svg)](https://crates.io/crates/geoq)

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

## Project Status

This library is still in its infancy and there are probably a lot of rough edges.

**What Works**

* The commands documented in the [Manual](https://github.com/worace/geoq/blob/master/manual.md) should mostly work
* Reading the supported input formats (Lat/Lon, Geohash, Wkt, Geojson) should be pretty reliable

**What Doesnt**

* Error handling is still pretty rough; in particular there's not great consistency between aborting loudly on some errors vs. handling and skipping over problematic inputs for others.
* Docs need improvement, and I'm especially hoping to add better usage instructions and examples to the built-in help texts.
* Some commands are still restricted to certain types of geometries
* Hopefully more features will be added soon, as well as potentially more supported input formats
