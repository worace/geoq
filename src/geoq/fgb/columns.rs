use flatbuffers::{FlatBufferBuilder, ForwardsUOffset, Vector, WIPOffset};
use flatgeobuf::Column;
use flatgeobuf::ColumnBuilder;

use super::header::ColSpec;

pub fn build<'a: 'b, 'b>(
    bldr: &'b mut FlatBufferBuilder<'a>,
    col_specs: &Vec<ColSpec>,
) -> WIPOffset<Vector<'a, ForwardsUOffset<Column<'a>>>> {
    let cols: Vec<WIPOffset<Column>> = col_specs
        .iter()
        .map(|c| {
            let col_name = bldr.create_string(&c.name);
            let mut cb = ColumnBuilder::new(bldr);
            cb.add_type_(c.type_);
            cb.add_name(col_name);
            cb.add_nullable(true);
            cb.finish()
        })
        .collect();
    bldr.create_vector(&cols[..])
}
