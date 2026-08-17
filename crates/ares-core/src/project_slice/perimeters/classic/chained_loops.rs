// Reached loop-construction seam from OrcaSlicer v2.4.2
// `PerimeterGenerator.cpp:208-210,227`. Entity traversal and orientation are deferred.

#[cfg(test)]
mod tests;
pub(in crate::project_slice) mod tree;
mod types;

#[cfg(test)]
pub(in crate::project_slice) use types::ExtrusionLoopRole;
#[cfg(test)]
pub(in crate::project_slice) use types::PreparedChainedLoopRecord as TestPreparedChainedLoopRecord;
pub(in crate::project_slice) use types::{
    ChainedLoopNode, ExtrusionLoop, PreparedChainedLoopObject, PreparedChainedLoopRecord,
    PreparedChainedLoopSurface, PreparedPostClassicChainedLoops,
};

use super::materialize::{
    PreparedPostClassicRawPaths, PreparedRawPathObject, PreparedRawPathRecord,
    PreparedRawPathSurface,
};

pub(in crate::project_slice) fn finish(
    prepared: PreparedPostClassicRawPaths,
) -> PreparedPostClassicChainedLoops {
    let PreparedPostClassicRawPaths {
        predecessor,
        objects,
    } = prepared;
    assert_eq!(objects.len(), predecessor.objects.len());
    let objects = objects
        .into_iter()
        .enumerate()
        .map(|(index, raw)| transform_object(raw, &predecessor.objects[index]))
        .collect();
    PreparedPostClassicChainedLoops {
        predecessor,
        objects,
    }
}

fn transform_object(
    raw: PreparedRawPathObject,
    traversal: &super::traversal::PostClassicTraversalPrintObject,
) -> PreparedChainedLoopObject {
    assert_eq!(raw.records.len(), traversal.records.len());
    let records = raw
        .records
        .into_iter()
        .enumerate()
        .map(|(index, raw)| match (raw, &traversal.records[index]) {
            (None, None) => None,
            (Some(raw), Some(traversal)) => Some(transform_record(raw, traversal)),
            _ => panic!("O5/O7 optional record alignment is invariant"),
        })
        .collect();
    PreparedChainedLoopObject { records }
}

fn transform_record(
    raw: PreparedRawPathRecord,
    traversal: &super::traversal::ClassicTraversalRecord,
) -> PreparedChainedLoopRecord {
    assert_eq!(raw.surfaces.len(), traversal.surfaces.len());
    let surfaces = raw
        .surfaces
        .into_iter()
        .enumerate()
        .map(|(index, raw)| transform_surface(raw, &traversal.surfaces[index], traversal.branch))
        .collect();
    PreparedChainedLoopRecord { surfaces }
}

fn transform_surface(
    raw: PreparedRawPathSurface,
    traversal: &super::traversal::PreparedTraversalSurface,
    branch: super::traversal::PendingPathBranch,
) -> PreparedChainedLoopSurface {
    assert_eq!(raw.source_index, traversal.source_index);
    let roots = tree::transform_nodes(raw.roots, &traversal.roots, branch);
    PreparedChainedLoopSurface {
        source_index: raw.source_index,
        roots,
    }
}
