//! Unsupported branch pruning in `TreeSupport.cpp:3133-3170`.

use std::collections::VecDeque;

use super::nodes::{NodeArena, NodeId};

pub(super) fn prune_unsupported(
    arena: &mut NodeArena,
    layers: &mut [Vec<NodeId>],
    mut unsupported: VecDeque<NodeId>,
) {
    while let Some(leaf) = unsupported.pop_back() {
        let mut current = Some(leaf);
        while let Some(id) = current {
            unlink(arena, id);
            arena.nodes[id].is_processed = true;
            for &neighbor in arena.nodes[id]
                .merged_neighbors
                .iter()
                .filter(|&&neighbor| !arena.nodes[neighbor].is_processed)
            {
                unsupported.push_front(neighbor);
            }
            current = arena.nodes[id].parent;
        }
    }
    for nodes in layers {
        nodes.retain(|&id| !arena.nodes[id].is_processed);
    }
}

fn unlink(arena: &mut NodeArena, id: NodeId) {
    let parent = arena.nodes[id].parent;
    let child = arena.nodes[id].child;
    if let Some(parent) = parent {
        arena.nodes[parent].child = child;
        for index in 0..arena.nodes[id].parents.len() {
            let parent = arena.nodes[id].parents[index];
            if arena.nodes[parent].child == Some(id) {
                arena.nodes[parent].child = child;
            }
        }
    }
    if let Some(child) = child {
        let (node, child_node) = if id < child {
            let (lower, upper) = arena.nodes.split_at_mut(child);
            (&lower[id], &mut upper[0])
        } else {
            let (lower, upper) = arena.nodes.split_at_mut(id);
            (&upper[0], &mut lower[child])
        };
        child_node.parent = parent;
        let slot = child_node
            .parents
            .iter()
            .position(|&parent| parent == id)
            .expect("a dropped node retains its parent in the child chain");
        child_node.parents.remove(slot);
        child_node.parents.extend_from_slice(&node.parents);
    }
}
