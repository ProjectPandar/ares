use super::super::{
    materialize::RawPathNode,
    shortest_path::chain_and_reorder_extrusion_paths,
    traversal::{PendingLoopRole, PendingPathBranch, TraversalSeed},
};
use super::types::{ChainedLoopNode, ExtrusionLoop, ExtrusionLoopRole};

struct FinishFrame {
    extrusion_loop: Option<ExtrusionLoop>,
    child_count: usize,
}

enum Frame<'a> {
    Visit(RawPathNode, &'a TraversalSeed),
    Finish(FinishFrame),
}

pub(super) fn transform_nodes(
    raw_nodes: Vec<RawPathNode>,
    seeds: &[TraversalSeed],
    branch: PendingPathBranch,
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
                let RawPathNode { paths, children } = raw;
                assert_eq!(children.len(), seed.children.len());
                let extrusion_loop = build_loop(paths, seed.loop_role, branch);
                frames.push(Frame::Finish(FinishFrame {
                    extrusion_loop,
                    child_count: children.len(),
                }));
                for (index, child) in children.into_iter().enumerate().rev() {
                    frames.push(Frame::Visit(child, &seed.children[index]));
                }
            }
            Frame::Finish(finish) => {
                let child_start = completed.len() - finish.child_count;
                let children = completed.split_off(child_start);
                completed.push(ChainedLoopNode {
                    extrusion_loop: finish.extrusion_loop,
                    children,
                });
            }
        }
    }
    completed
}

fn build_loop(
    mut paths: Vec<super::super::materialize::ExtrusionPath>,
    role: PendingLoopRole,
    branch: PendingPathBranch,
) -> Option<ExtrusionLoop> {
    if matches!(branch, PendingPathBranch::OverhangClipping { .. }) {
        let first = paths.first()?.polyline.points[0];
        chain_and_reorder_extrusion_paths(&mut paths, [first.x, first.y]);
    }
    Some(ExtrusionLoop {
        paths,
        role: map_loop_role(role),
    })
}

fn map_loop_role(role: PendingLoopRole) -> ExtrusionLoopRole {
    match role {
        PendingLoopRole::Internal => ExtrusionLoopRole::Internal,
        PendingLoopRole::Default => ExtrusionLoopRole::Default,
        PendingLoopRole::Hole => ExtrusionLoopRole::Hole,
    }
}

#[cfg(test)]
pub(in crate::project_slice) fn consume_nodes(mut nodes: Vec<ChainedLoopNode>) {
    while let Some(mut node) = nodes.pop() {
        if let Some(extrusion_loop) = node.extrusion_loop {
            for path in extrusion_loop.paths {
                let _ = (
                    path.polyline.points,
                    path.role,
                    path.mm3_per_mm,
                    path.width,
                    path.height,
                );
            }
            let _ = extrusion_loop.role;
        }
        nodes.append(&mut node.children);
    }
}
