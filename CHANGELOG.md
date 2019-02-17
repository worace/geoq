## Changelog

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
