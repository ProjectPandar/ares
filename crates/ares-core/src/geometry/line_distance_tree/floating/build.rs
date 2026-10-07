//! `AABBTreeIndirect.hpp:125-215::build_recursive/partition_input`, native precision.

use super::{Bounds, NativeFloat, Node, NodeKind};

#[derive(Clone, Copy)]
struct Input<F> {
    index: usize,
    bounds: Bounds<F>,
    centroid: [F; 2],
}

pub(super) fn build<F: NativeFloat>(lines: &[[[F; 2]; 2]]) -> Vec<Node<F>> {
    if lines.is_empty() {
        return Vec::new();
    }
    let mut input = lines
        .iter()
        .enumerate()
        .map(|(index, &[a, b])| Input {
            index,
            bounds: Bounds {
                min: [a[0].min(b[0]), a[1].min(b[1])],
                max: [a[0].max(b[0]), a[1].max(b[1])],
            },
            centroid: [(a[0] + b[0]) * F::HALF, (a[1] + b[1]) * F::HALF],
        })
        .collect::<Vec<_>>();
    let mut nodes = vec![
        Node {
            kind: NodeKind::Unused,
            bounds: Bounds {
                min: [F::ZERO; 2],
                max: [F::ZERO; 2]
            },
        };
        input.len().next_power_of_two() * 2 - 1
    ];
    build_recursive(&mut input, &mut nodes, 0);
    nodes
}

fn build_recursive<F: NativeFloat>(input: &mut [Input<F>], nodes: &mut [Node<F>], index: usize) {
    if input.len() == 1 {
        nodes[index] = Node {
            kind: NodeKind::Leaf(input[0].index),
            bounds: input[0].bounds,
        };
        return;
    }
    let mut bounds = input[0].bounds;
    for item in &input[1..] {
        for axis in 0..2 {
            bounds.min[axis] = bounds.min[axis].min(item.bounds.min[axis]);
            bounds.max[axis] = bounds.max[axis].max(item.bounds.max[axis]);
        }
    }
    let axis = usize::from(bounds.max[1] - bounds.min[1] > bounds.max[0] - bounds.min[0]);
    let middle = (input.len() - 1) / 2;
    partition(input, axis, middle);
    nodes[index] = Node {
        kind: NodeKind::Inner,
        bounds,
    };
    let (left, right) = input.split_at_mut(middle + 1);
    build_recursive(left, nodes, index * 2 + 1);
    build_recursive(right, nodes, index * 2 + 2);
}

fn partition<F: NativeFloat>(input: &mut [Input<F>], axis: usize, k: usize) {
    let mut left = 0;
    let mut right = input.len() - 1;
    while left < right {
        let middle = (left + right) / 2;
        let mut left_value = input[left].centroid[axis];
        let mut middle_value = input[middle].centroid[axis];
        let mut right_value = input[right].centroid[axis];
        if left_value > middle_value {
            input.swap(left, middle);
            std::mem::swap(&mut left_value, &mut middle_value);
        }
        if left_value > right_value {
            input.swap(left, right);
            right_value = left_value;
        }
        if middle_value > right_value {
            input.swap(middle, right);
            middle_value = right_value;
        }
        let pivot = middle_value;
        if right <= left + 2 {
            break;
        }
        let mut i = left;
        let mut j = right - 1;
        input.swap(middle, j);
        loop {
            i += 1;
            while input[i].centroid[axis] < pivot {
                i += 1;
            }
            j -= 1;
            while input[j].centroid[axis] > pivot && i < j {
                j -= 1;
            }
            if i >= j {
                break;
            }
            input.swap(i, j);
        }
        input.swap(i, right - 1);
        if k < i {
            right = i - 1;
        } else if k == i {
            break;
        } else {
            left = i + 1;
        }
    }
}
