//! Input ordering from `AABBTreeIndirect.hpp::build_recursive/partition_input`.

use super::super::layer_growth::SupportElementMerging;
use crate::geometry::Coord;

pub(super) fn build_recursive(nodes: &mut [SupportElementMerging]) {
    if nodes.len() == 1 {
        return;
    }
    let mut bbox = nodes[0].bbox.expect("grown merge node has a bbox");
    for node in &nodes[1..] {
        let next = node.bbox.expect("grown merge node has a bbox");
        bbox.merge_point(next.min());
        bbox.merge_point(next.max());
    }
    let dimension = usize::from(bbox.max().y() - bbox.min().y() > bbox.max().x() - bbox.min().x());
    let center = (nodes.len() - 1) / 2;
    partition_input(nodes, dimension, center);
    let (left, right) = nodes.split_at_mut(center + 1);
    build_recursive(left);
    build_recursive(right);
}

fn centroid(node: &SupportElementMerging, dimension: usize) -> Coord {
    let center = node.bbox.expect("grown merge node has a bbox").center();
    if dimension == 0 {
        center.x()
    } else {
        center.y()
    }
}

fn partition_input(nodes: &mut [SupportElementMerging], dimension: usize, k: usize) {
    let mut left = 0;
    let mut right = nodes.len() - 1;
    while left < right {
        let center = (left + right) / 2;
        let mut left_value = centroid(&nodes[left], dimension);
        let mut center_value = centroid(&nodes[center], dimension);
        let mut right_value = centroid(&nodes[right], dimension);
        if left_value > center_value {
            nodes.swap(left, center);
            std::mem::swap(&mut left_value, &mut center_value);
        }
        if left_value > right_value {
            nodes.swap(left, right);
            right_value = left_value;
        }
        if center_value > right_value {
            nodes.swap(center, right);
            center_value = right_value;
        }
        let pivot = center_value;
        if right <= left + 2 {
            break;
        }
        let mut i = left;
        let mut j = right - 1;
        nodes.swap(center, j);
        loop {
            i += 1;
            while centroid(&nodes[i], dimension) < pivot {
                i += 1;
            }
            j -= 1;
            while centroid(&nodes[j], dimension) > pivot && i < j {
                j -= 1;
            }
            if i >= j {
                break;
            }
            nodes.swap(i, j);
        }
        nodes.swap(i, right - 1);
        if k < i {
            right = i - 1;
        } else if k == i {
            break;
        } else {
            left = i + 1;
        }
    }
}
