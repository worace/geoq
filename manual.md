# Geoq Usage Manual

## Input Formats

For most commands, geoq accepts linewise input via STDIN. The following common GIS formats are accepted:

* WKT
* GeoJSON
* Geohashes (Base 32 encoded, e.g. `9q5`)
* Comma or Tab-separated Latitude/Longitude Pairs: `12.0,34.0` or `12.0	34.0`

Remember that even for WKT or GeoJSON inputs, they must be submitted **1 per line**. [jq](https://stedolan.github.io/jq/) can be useful for compacting unruly GeoJSON inputs if needed: `cat multi_line_geojsons.json | jq -cr . | geoq ...`.

### Note on Feature Collections

GeoJSON includes a [Feature Collection](https://macwright.org/2015/03/23/geojson-second-bite.html#featurecollection) type which represents a collection of multiple GeoJSON features.

When reading a Feature Collection, geoq will actually unwrap the collection and treat its features as individual inputs, e.g.:

```
echo '{"features":[{"geometry":{"coordinates":[-118.44,34.7],"type":"Point"},"properties":{},"type":"Feature"},{"geometry":{"coordinates":[-117.87,35.06],"type":"Point"},"properties":{},"type":"Feature"}],"type":"FeatureCollection"}' | geoq wkt
POINT(-118.44 34.7)
POINT(-117.87 35.06)
```

This behaves the same as if you had provided each point as its own GeoJSON feature on its own line in absence of the Feature Collection.

This is useful for exploding and manipulating individual features, and it means that `geoq gj fc` can even be used to concat multiple feature collections. If you actually want to have a collection of Lines/Polygons/Points treated as a single geometry, try one of the [Multi- Geometry variants](https://macwright.org/2015/03/23/geojson-second-bite.html#multi-geometries).

## Commands

### GeoJSON - `geoq gj`

#### As Geometry: `geoq gj geom`

```
echo 9q5 | geqo gj geom
{"coordinates":[[[-119.53125,33.75],[-118.125,33.75],[-118.125,35.15625],[-119.53125,35.15625],[-119.53125,33.75]]],"type":"Polygon"}
```

#### As Feature: `geoq gj f`

```
echo 9q5 | geoq gj f
{"geometry":{"coordinates":[[[-119.53125,33.75],[-118.125,33.75],[-118.125,35.15625],[-119.53125,35.15625],[-119.53125,33.75]]],"type":"Polygon"},"properties":{},"type":"Feature"}
```

#### As FeatureCollection: `geoq gj fc`

Note: This command behaves as an aggregation -- it will gather all inputs provided via STDIN and combine them into a single GeoJSON Feature Collection.

```
print "12,34\n56,78\n" | geoq gj fc | jq
{
  "features": [
    {
      "geometry": {
        "coordinates": [
          34,
          12
        ],
        "type": "Point"
      },
      "properties": {},
      "type": "Feature"
    },
    {
      "geometry": {
        "coordinates": [
          78,
          56
        ],
        "type": "Point"
      },
      "properties": {},
      "type": "Feature"
    }
  ],
  "type": "FeatureCollection"
}
```

### WKT - `geoq wkt`

Output each entity as WKT:

```
echo 9q5 | geoq wkt
POLYGON((-119.53125 33.75,-118.125 33.75,-118.125 35.15625,-119.53125 35.15625,-119.53125 33.75))
```

### Geohashes - `geoq gh`

#### Geohash for a point - `geoq gh point`

Get the Base 32 geohash representation for a point at a given character level.

Note that only Point geometries are supported for this operation.

```
echo 12,34 | geoq gh point 3
sf0
echo '{"coordinates":[34.0,12.0],"type":"Point"}' | geoq gh point 3
sf0
```

#### Geohash neighbors - `geoq gh neighbors`

Get the adjacent geohashes for a given geohash.

Only geohash inputs are supported.

```
echo 9q5 | geoq gh neighbors
9q5
9q7
9qk
9qh
9mu
9mg
9mf
9q4
9q6
```

Note: By default, the given geohash is also included in the output, giving a full 3x3 grid centered on the given geohash. **To reverse this behavior, use the `-e` flag.**

```
echo 9q5 | geoq gh neighbors -e
9q7
9qk
9qh
9mu
9mg
9mf
9q4
9q6
```

#### Geohash children - `geoq gh children`

```
cho 9q5 | geoq gh children
9q50
9q51
9q52
9q53
9q54
9q55
9q56
(...)
```

#### Covering geohashes - `geoq gh covering`

Get the set of geohashes "covering" each provided entity. This is useful for doing geohash tiling of geometries.


```
echo 'POLYGON((-86.30 32.37,-86.33 32.36,-86.30 32.34,-86.28 32.35,-86.30 32.37))' | geoq gh covering 5
djf8h
djf85
djdxu
```

It's also possible to include the original input with its list of geohashes. Use the `-o` flag for this:

```
echo 'POLYGON((-86.30 32.37,-86.33 32.36,-86.30 32.34,-86.28 32.35,-86.30 32.37))' | geoq gh covering 5 -o
POLYGON((-86.30 32.37,-86.33 32.36,-86.30 32.34,-86.28 32.35,-86.30 32.37))
djf8h
djf85
djdxu
```

### Mapping - `geoq map`

Display given entities on a map using [geojson.io](http://geojson.io).

```
echo 9q5 | geoq map
```

This involves a 2-step process where geoq first aggregates all your inputs into a GeoJSON Feature Collection (similar to `geoq gj fc`), and then sends the result to geojson.io in your browser via a URL parameter.

Unfortunately, there is a limit on the amount of data that can be sent to geojson.io this way, so larger inputs will not work. We hope to have a better solution for this in the future but for now GeoJSON data over 27k characters will be rejected.

### Filtering - `geoq filter`

Select geometries which match certain conditions

#### Filter by Polygon intersection: `geoq filter intersects <query>`

Will output only those entities which intersect the given query geometry.

Currently only polygons are supported for querying, but hopefully more geometries will be added soon.

```
print "34.70,-118.44\n35.06,-117.87\n" | geoq filter intersects 9q5
34.70,-118.44
```

### Flatgeobuf - `geoq fgb`

#### Writing Flatgeobuf Files

`geoq fgb write <FILE>` takes newline-delimited GeoJSON input from STDIN and writes a flatgeobuf in the specified output file

```
echo '{"coordinates":[-123.1874,48.7902],"type":"Point"}' | \
geoq fgb write /tmp/point.fgb
```

#### Reading Flatgeobuf

`geoq fgb read <FILE>` reads flatgeobuf files and prints rows as GeoJSON to STDOUT.

Accepts optional `--bbox` arg for filtering a bounding box using the fgb index. BBox should be specified as `min_x,min_y,max_x,max_y` e.g. `--bbox -123.2,48.8,-123.1,48.7`

```
geoq fgb read /tmp/point.fgb
```
