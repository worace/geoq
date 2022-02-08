use std::ops::Range;

use super::hilbert::BBox;
use super::hilbert::BoundedFeature;

pub const NODE_SIZE: u16 = 16;

pub struct IndexNode {
    bbox: BBox,
    featureByteOffset: usize,
}

fn write_index(hilbert_sorted_features: Vec<BoundedFeature>, extent: BBox) -> Vec<u8> {
    // 1. determine level bounds based on num features
    // 2. allocate buffer for nodes
    // 3. fill in intermediate nodes
    //    - allocate nodes
    //    - populate bboxes (???)
    // 4. fill in leaf nodes
    let tree_structure = calculate_level_bounds(hilbert_sorted_features.len());
    let mut index_nodes: Vec<IndexNode> = Vec::with_capacity(tree_structure.num_nodes);
    // Q: how to set the bbox bounds for the intermediate nodes?
    //  - build from bottom up?
    //  - leaf node 0 - 15 to LN-1 node 0
    //  - leaf node 16 - 32 to LN-2 node 1
    vec![]
}

pub struct RTreeIndexMeta {
    num_features: usize,
    num_nodes: usize,
    num_nodes_per_level: Vec<usize>,
    level_bounds: Vec<Range<usize>>,
}

// Statically calculate the structure of the tree required
// to hold the specified number of nodes.
// The total number of nodes will be the number of features
// plus however many upper-level nodes are needed to
// represent the required amount of nesting
fn calculate_level_bounds(num_features: usize) -> RTreeIndexMeta {
    let node_size = NODE_SIZE as usize;

    let mut nodes_per_level: Vec<usize> = vec![];
    let mut current_level_size = num_features;
    loop {
        nodes_per_level.push(current_level_size);

        let next_level_size = if current_level_size % node_size == 0 {
            current_level_size / node_size
        } else {
            current_level_size / node_size + 1
        };

        if next_level_size == 1 {
            nodes_per_level.push(next_level_size);
            break;
        } else {
            current_level_size = next_level_size;
        }
    }
    nodes_per_level.reverse();
    let mut nodes_so_far = 0;
    let mut level_bounds: Vec<Range<usize>> = vec![];
    for num_nodes in nodes_per_level.iter() {
        let end = num_nodes + nodes_so_far;
        level_bounds.push(nodes_so_far..end);
        nodes_so_far = end;
    }
    RTreeIndexMeta {
        num_features: num_features,
        num_nodes: nodes_per_level.iter().sum(),
        num_nodes_per_level: nodes_per_level,
        level_bounds: level_bounds,
    }
}

#[test]
fn test_level_bounds() {
    let a = calculate_level_bounds(179);
    assert_eq!(a.num_features, 179);
    assert_eq!(a.num_nodes, 192);
    assert_eq!(a.num_nodes_per_level, vec![1, 12, 179]);
    assert_eq!(a.level_bounds, vec![0..1, 1..13, 13..192]);

    let b = calculate_level_bounds(15);
    assert_eq!(b.num_features, 15);
    assert_eq!(b.num_nodes, 16);
    assert_eq!(b.num_nodes_per_level, vec![1, 15]);
    assert_eq!(b.level_bounds, vec![0..1, 1..16]);

    let c = calculate_level_bounds(100000);
    assert_eq!(c.num_features, 100000);
    assert_eq!(c.num_nodes, 106669);
    assert_eq!(c.num_nodes_per_level, vec![1, 2, 25, 391, 6250, 100000]);
    assert_eq!(
        c.level_bounds,
        vec![0..1, 1..3, 3..28, 28..419, 419..6669, 6669..106669]
    );
}
