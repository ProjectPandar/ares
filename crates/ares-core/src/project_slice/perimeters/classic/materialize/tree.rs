use crate::{SliceError, geometry::CoordinateScale};

use super::{
    super::traversal::{
        ClassicTraversalRecord, PostClassicTraversalPrintObject, PreparedTraversalSurface,
        TraversalSeed,
    },
    path::materialize_seed,
    types::{PreparedRawPathSurface, RawPathNode},
};

enum Frame<'a> {
    Visit(&'a TraversalSeed),
    Finish(&'a TraversalSeed),
}

pub(super) fn materialize_surface(
    object: &PostClassicTraversalPrintObject,
    record_index: usize,
    record: &ClassicTraversalRecord,
    surface: &PreparedTraversalSurface,
    scale: CoordinateScale,
) -> Result<PreparedRawPathSurface, SliceError> {
    let mut frames = Vec::new();
    frames.extend(surface.roots.iter().rev().map(Frame::Visit));
    let mut completed = Vec::new();
    while let Some(frame) = frames.pop() {
        match frame {
            Frame::Visit(seed) => {
                frames.push(Frame::Finish(seed));
                frames.extend(seed.children.iter().rev().map(Frame::Visit));
            }
            Frame::Finish(seed) => {
                let child_start = completed.len() - seed.children.len();
                let children = completed.split_off(child_start);
                let paths = match materialize_seed(object, record_index, record, seed, scale) {
                    Ok(paths) => paths,
                    Err(error) => {
                        consume_nodes(children);
                        consume_nodes(completed);
                        return Err(error);
                    }
                };
                completed.push(RawPathNode { paths, children });
            }
        }
    }
    Ok(PreparedRawPathSurface {
        source_index: surface.source_index,
        roots: completed,
    })
}

pub(in crate::project_slice) fn consume_nodes(mut nodes: Vec<RawPathNode>) {
    while let Some(mut node) = nodes.pop() {
        for path in node.paths {
            let _ = (
                path.polyline.points,
                path.role,
                path.mm3_per_mm,
                path.width,
                path.height,
            );
        }
        nodes.append(&mut node.children);
    }
}
