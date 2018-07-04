## TODO

* [X] Try assert cli for testing CLI "type" command https://github.com/assert-rs/assert_cli
* [X] Add rust-geohash and generate point geohashes
* [X] test reader matching functions
* [X] Geojson geom printing
* [X] Geojson feature printing
* [ ] Geojson feature collection aggregation
* [ ] Add wkt writing -- perhaps borrow from https://github.com/CanalTP/rust-wkt/tree/write-wkt ?
* [ ] Preserve features on existing GJ features
* [ ] Point geohash (`echo 12,34 | geoq gh point 4 => abcd`)
* [ ] map via geojson.io
* [ ] Map large JSON with embedded copy of geojson.io web page
* [ ] Refactor reading interface to produce an iter<results>
* [ ] Migrate Geojson and WKT output formats to trait
* [ ] Add strings in error messages so, e.g. bad inputs can be printed with error
* [ ] Investigate parallel iterator processing (rayon?)
* [ ] Try streaming serde for feature collections: https://github.com/serde-rs/json/issues/345
* [ ] `filter` subcommand
  * intersects
  * contains
