use std::ops::Range;

use super::hilbert::BBox;
use super::hilbert::BoundedFeature;
use super::hilbert::IndexNode;

pub const NODE_SIZE: u16 = 16;
// 4 doubles for bbox + 1 u64 for byte offset
pub const NODE_STORAGE_BYTES: usize = 40;

pub fn build_flattened_tree(
    hilbert_sorted_features: Vec<IndexNode>,
    _extent: &BBox,
) -> (RTreeIndexMeta, Vec<IndexNode>) {
    // 1. determine level bounds based on num features
    // 2. allocate buffer for nodes
    // 3. fill in intermediate nodes
    //    - allocate nodes
    //    - populate bboxes (???)
    // 4. fill in leaf nodes
    let tree_structure = calculate_level_bounds(hilbert_sorted_features.len());
    let placeholder_node = IndexNode {
        bbox: BBox::empty(),
        offset: 0,
    };
    let mut flattened_tree: Vec<IndexNode> = vec![placeholder_node; tree_structure.num_nodes];
    eprintln!("Allocated len for index nodes: {:?}", flattened_tree.len());

    eprintln!("tree: {:?}", tree_structure);
    let bottom = tree_structure
        .level_bounds
        .last()
        .expect("Expecting at least 2 levels in tree");

    // Populate the bottom tier of the tree which makes up the last section
    // of the flattened index buffer. The index nodes here contain byte offsets
    // into the features section of the tree, and the node positions are index offsets
    // based on the calculated level hierarchy layout
    eprintln!("iter bottom tree level - {:?}", bottom);
    for (feature_index, node_index) in bottom.clone().enumerate() {
        flattened_tree[node_index] = hilbert_sorted_features[feature_index].clone();
    }

    // iterate non-leaf levels from bottom up
    // iterate this level's nodes, for each one,
    // consider the sub-slice of the previous-level's nodes which are covered by it
    // (0..NODE_SIZE)
    // and expand this nodes bbox by that ones
    // L0: 0..1
    // L1: 1..13
    // L2: 13..192
    for (level_index, level_bounds) in tree_structure.level_bounds.iter().enumerate().rev().skip(1)
    {
        eprintln!(
            "iterate non-leaf level: {:?} - {:?}",
            level_index, level_bounds
        );
        let prev_level = tree_structure.level_bounds[level_index + 1].clone();

        for node_index in level_bounds.clone() {
            let mut bbox: Option<BBox> = None;
            let prev_level_slice_start = prev_level.start + node_index * NODE_SIZE as usize;
            let prev_level_slice_end = prev_level.start + (node_index + 1) * NODE_SIZE as usize;

            for prev_idx in prev_level_slice_start..prev_level_slice_end {
                if prev_idx > prev_level.len() {
                    break;
                }
                // eprintln!(
                //     "populate data from index {:?} in prev level into index {:?} in current",
                //     prev_idx, node_index,
                // );
                // eprintln!(
                //     "expand current bbox: {:?} from {:?}",
                //     bbox, &flattened_tree[prev_idx].bbox
                // );
                if let Some(ref mut bb) = bbox {
                    bb.expand(&flattened_tree[prev_idx].bbox)
                } else {
                    bbox = Some(flattened_tree[prev_idx].bbox.clone());
                }
            }

            // Offset for non-leaf index nodes whould be the index where its set of N (node_size) nodes starts
            let node = IndexNode {
                bbox: bbox.unwrap_or(BBox::empty()),
                offset: prev_level_slice_start,
            };
            flattened_tree[node_index] = node;
        }
    }

    (tree_structure, flattened_tree)
}

pub fn serialize(flattened_tree: Vec<IndexNode>) -> Vec<u8> {
    let size = flattened_tree.len() * NODE_STORAGE_BYTES;
    let mut buf: Vec<u8> = Vec::with_capacity(size);
    for node in flattened_tree {
        eprintln!(
            "(Geoq) Write Node {:?}, {:?}, {:?}, {:?}, {:?}",
            node.bbox.min_x, node.bbox.min_y, node.bbox.max_x, node.bbox.max_y, node.offset,
        );
        buf.extend(node.bbox.min_x.to_le_bytes());
        buf.extend(node.bbox.min_y.to_le_bytes());
        buf.extend(node.bbox.max_x.to_le_bytes());
        buf.extend(node.bbox.max_y.to_le_bytes());
        buf.extend(node.offset.to_le_bytes());
    }
    buf
}

#[derive(Debug)]
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
        num_features,
        num_nodes: nodes_per_level.iter().sum(),
        num_nodes_per_level: nodes_per_level,
        level_bounds,
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

#[test]
fn test_building_index() {
    let nodes = vec![
        IndexNode {
            bbox: BBox {
                min_x: 11.0,
                min_y: -29.0,
                max_x: 25.0,
                max_y: -16.0,
            },
            offset: 0,
        },
        IndexNode {
            bbox: BBox {
                min_x: 16.0,
                min_y: -34.0,
                max_x: 32.0,
                max_y: -22.0,
            },
            offset: 100,
        },
    ];
    let extent = BBox {
        min_x: 11.0,
        min_y: -34.0,
        max_x: 32.0,
        max_y: -16.0,
    };
    let idx = build_flattened_tree(nodes.clone(), &extent);

    assert_eq!(&extent, &idx.1[0].bbox);
    assert_eq!(&nodes[0].bbox, &idx.1[1].bbox);
    assert_eq!(&nodes[1].bbox, &idx.1[2].bbox);

    assert_eq!(idx.0.num_features, 2);
    assert_eq!(idx.0.num_nodes, 3);
    assert_eq!(idx.0.level_bounds.len(), 2);
    assert_eq!(idx.0.num_nodes_per_level, vec![1, 2]);
}
