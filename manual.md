# Geoq Usage Manual

## Input Formats

For most commands, geoq accepts linewise input via STDIN. The following common GIS formats are accepted:

* WKT
* GeoJSON
* Geohashes (Base 32 encoded, e.g. `9q5`)
* Comma or Tab-separated Latitude/Longitude Pairs: `12.0,34.0` or `12.0	34.0`

Remember that even for WKT or GeoJSON inputs, they must be submitted **1 per line**. [jq](https://stedolan.github.io/jq/** can be useful for compacting unruly GeoJSON inputs if needed: `cat multi_line_geojsons.json | jq -cr . | geoq ...`.

## Commands

### GeoJSON - `geoq gj`

**As Geometry: `geoq gj geom`**

```
echo 9q5 | geqo gj geom
{"coordinates":[[[-119.53125,33.75],[-118.125,33.75],[-118.125,35.15625],[-119.53125,35.15625],[-119.53125,33.75]]],"type":"Polygon"**
```

**As Feature: `geoq gj f`**

```
echo 9q5 | geoq gj f
{"geometry":{"coordinates":[[[-119.53125,33.75],[-118.125,33.75],[-118.125,35.15625],[-119.53125,35.15625],[-119.53125,33.75]]],"type":"Polygon"},"properties":{},"type":"Feature"**
```

**As FeatureCollection: `geoq gj fc`**

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
POLYGON((-119.53125 33.75,-118.125 33.75,-118.125 35.15625,-119.53125 35.15625,-119.53125 33.75)**
```

### Geohashes - `geoq gh`

**Get the geohash for a point at a given level:**

Note that only Point geometries are supported for this operation.

```
echo 12,34 | geoq gh point 3
sf0
echo '{"coordinates":[34.0,12.0],"type":"Point"}** | geoq gh point 3
sf0
```

**Get neighbors for a given geohash**

Only geohashes are supported.

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

**Get children for a given geohash**

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

**Get the set of geohashes "covering" a given geometry**

```
echo 'POLYGON((-86.30 32.37,-86.33 32.36,-86.30 32.34,-86.28 32.35,-86.30 32.37))' | geoq gh covering 5
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

Unfortunately, there is a limit on the amount of data that can be sent to geojson.io this way, so larger inputs will not work.

### Filtering - `geoq filter`
