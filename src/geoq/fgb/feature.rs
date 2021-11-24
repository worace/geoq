use super::columns;
use super::geometry;
use super::header::ColSpec;
use super::properties;
use flatbuffers::FlatBufferBuilder;

// table Feature {
//   geometry: Geometry;  // Geometry
//   properties: [ubyte]; // Custom buffer, variable length collection of key/value pairs (key=ushort)
//   columns: [Column];   // Attribute columns schema (optional)
// }
pub fn write<'a>(col_specs: &Vec<ColSpec>, f: &geojson::Feature) -> FlatBufferBuilder<'a> {
    let mut bldr = FlatBufferBuilder::new();
    // eprintln!("Write geojson feature: {:?}", f);
    // https://github.com/flatgeobuf/flatgeobuf/blob/master/src/ts/generic/feature.ts#L47-L143
    // flatgeobuf::GeometryOffset

    // Q: should this repeat all columns for the schema, or only the ones that apply to this feature?
    let cols_vec = columns::build(&mut bldr, col_specs);
    dbg!(col_specs);
    let props = properties::feature_props(f, col_specs).map(|bytes| bldr.create_vector(&bytes[..]));

    // Geometry serialization
    // https://github.com/flatgeobuf/flatgeobuf/blob/master/src/ts/generic/geometry.ts#L37-L64
    let geom = geometry::build(&mut bldr, f);

    let args = flatgeobuf::FeatureArgs {
        columns: None,
        geometry: Some(geom),
        properties: None,
    };
    let offset = flatgeobuf::Feature::create(&mut bldr, &args);

    bldr.finish_size_prefixed(offset, None);
    bldr
}
