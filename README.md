# geoq

[![Build Status](https://travis-ci.org/worace/geoq.svg?branch=master)](https://travis-ci.org/worace/geoq)

[![crates.io](https://img.shields.io/badge/crates.io-v0.0.1-orange.svg)](https://crates.io/crates/geoq)

Geoq is a command-line tool for working with geospatial data.

It unifies a few common GIS serialization formats with come utilities for manipulating and visualizing data on a map.

As a brief taste, the command:

```
echo '{"type":"Polygon","coordinates":[[[30,10],[40,40],[20,40],[10,20],[30,10]]]}' | geoq map
```

will open a browser window to render the desired polygon on a map.

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
