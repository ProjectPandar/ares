use super::super::{
    materialize::RawPathNode, traversal::PendingLoopRole, traversal::TraversalSeed,
};

use super::types::{ChainedLoopNode, ExtrusionLoop, ExtrusionLoopRole};

enum Frame<'a> {
    Visit(RawPathNode, &'a TraversalSeed),
    Finish(usize),
}

pub(super) fn transform_nodes(
    raw_nodes: Vec<RawPathNode>,
    seeds: &[TraversalSeed],
) -> Vec<ChainedLoopNode> {
    assert_eq!(raw_nodes.len(), seeds.len());
    let mut frames = Vec::new();
    for (index, raw) in raw_nodes.into_iter().enumerate().rev() {
        frames.push(Frame::Visit(raw, &seeds[index]));
    }
    let mut completed = Vec::new();
    while let Some(frame) = frames.pop() {
        match frame {
            Frame::Visit(raw, seed) => {
                let RawPathNode {
                    paths,
                    children,
                    child_order,
                } = raw;
                assert_eq!(children.len(), seed.children.len());
                // Empty paths keep the `None` slot for source line 208
                // `continue` — the loop is not an entity and compacts out of
                // the emission chain. The overhang path reorder already ran
                // in the materialization flat pass (`:207`).
                let extrusion_loop = (!paths.is_empty()).then_some(ExtrusionLoop {
                    paths,
                    role: map_loop_role(seed.loop_role),
                    print_after_infill: false,
                });
                let child_count = children.len();
                frames.push(Frame::Finish(child_count));
                for (index, child) in children.into_iter().enumerate().rev() {
                    frames.push(Frame::Visit(child, &seed.children[index]));
                }
                completed.push(ChainedLoopNode {
                    extrusion_loop,
                    children: Vec::new(),
                    child_order,
                });
            }
            Frame::Finish(child_count) => {
                let child_start = completed.len() - child_count;
                let children = completed.split_off(child_start);
                completed
                    .last_mut()
                    .expect("a finish has its node")
                    .children = children;
            }
        }
    }
    completed
}

fn map_loop_role(role: PendingLoopRole) -> ExtrusionLoopRole {
    match role {
        PendingLoopRole::Internal => ExtrusionLoopRole::Internal,
        PendingLoopRole::Default => ExtrusionLoopRole::Default,
        PendingLoopRole::Hole => ExtrusionLoopRole::Hole,
    }
}
