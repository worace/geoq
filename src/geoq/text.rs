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
