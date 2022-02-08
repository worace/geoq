use std::ops::Range;

use super::hilbert::BBox;
use super::hilbert::BoundedFeature;

pub const NODE_SIZE: u16 = 16;

fn write_index(hilbert_sorted_features: Vec<BoundedFeature>, extent: BBox) -> Vec<u8> {
    // 1. determine level bounds based on num features
    // 2.
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
    // let mut num_upper_levels: u32 = 1; // need at least the root level
    let node_size = NODE_SIZE as usize;
    // while node_size.pow(num_upper_levels) < num_features {
    //     num_upper_levels += 1;
    // }

    // 179, 12, 1
    // let num_levels = num_upper_levels + 1;

    let mut nodes_per_level: Vec<usize> = vec![];
    let mut current_level_size = num_features;
    loop {
        nodes_per_level.push(current_level_size);

        eprintln!(
            "check next level up from level of size {:?}",
            current_level_size
        );
        // stop at root level when we reach 1 node

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

        eprintln!("next level size: {:?}", current_level_size);
    }
    nodes_per_level.reverse();
    dbg!(&nodes_per_level);

    // level_num_nodes: contains numer of nodes per level starting
    //   with the bottom level, which contains only leaf nodes, i.e.
    //   individual features
    // n: goes from total number of features (bottom level)
    //    up to 1, reducing per level by factor of node_size
    //    e.g.
    //    n = 179
    //    n = (179 + (16 - 1)) / 16 = 12 --> takes 12 16-item tree nodes to hold 179 leaf items
    //    level_num_nodes = [179, 12];
    //    -----iter 2
    //    n = (12 + (16 - 1)) / 16 = 1
    //    level_num_nodes = [179, 12, 1];
    //      * reached n == 1, so stop here. this is our tree layout
    //               <root>
    //       /1         |2      ...     \12
    // /1.1...\1.16   /2.1...\2.16     /12.1...\12.3
    //                                 (12 * 16 - 179 = 13)
    //                                 13 slots open in last tree node

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
