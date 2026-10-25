use crate::{
    SliceError,
    geometry::{Coord, CoordinateScale, ThickPolyline},
};

use super::{
    super::{
        entity_collections::{ExtrusionEntity, loop_endpoints},
        gap_extrusion::variable_width::convert_entities,
        shortest_path::{
            chain_extrusion_entities, chain_extrusion_paths, chain_loops_by_first_points,
        },
        traversal::{
            ClassicTraversalRecord, PendingPathBranch, PostClassicTraversalPrintObject,
            PreparedTraversalSurface, TraversalSeed,
        },
    },
    path::materialize_seed,
    types::{ExtrusionPath, ExtrusionRole, PreparedRawPathSurface, RawPathNode},
};

/// One `traverse_loops` recursion level (`PerimeterGenerator.cpp:142-151`
/// flat pass + `:254-255` child descent). The upstream flat pass fuzzifies
/// every same-level sibling in `loops` order before any child recursion, so
/// the shared seeded stream is consumed sibling-first while the emitted
/// `RawPathNode` tree keeps the children-first structure the emission
/// traversal (`:274-278`) expects.
struct LevelFrame<'a> {
    seeds: &'a [TraversalSeed],
    /// Thin walls chain with the root level only (`PerimeterGenerator.cpp:230-233`).
    thin_walls: Option<&'a [ThickPolyline]>,
    flat_done: bool,
    paths: Vec<Vec<ExtrusionPath>>,
    /// Child visit order: chain-ordered loop slots, then the compacted-out
    /// tail slots in vector order.
    visits: Vec<usize>,
    next_visit: usize,
    pending_slot: Option<usize>,
    children: Vec<Vec<RawPathNode>>,
}

impl<'a> LevelFrame<'a> {
    fn new(seeds: &'a [TraversalSeed], thin_walls: Option<&'a [ThickPolyline]>) -> Self {
        Self {
            seeds,
            thin_walls,
            flat_done: false,
            paths: Vec::with_capacity(seeds.len()),
            visits: Vec::new(),
            next_visit: 0,
            pending_slot: None,
            children: std::iter::repeat_with(Vec::new).take(seeds.len()).collect(),
        }
    }
}

pub(super) fn materialize_surface(
    object: &PostClassicTraversalPrintObject,
    record_context: super::RecordContext<'_>,
    surface: &PreparedTraversalSurface,
    scale: CoordinateScale,
    context: &mut crate::SlicerContext,
) -> Result<PreparedRawPathSurface, SliceError> {
    let record = record_context.record;
    let mut frames = vec![LevelFrame::new(&surface.roots, Some(&surface.thin_walls))];
    let mut completed: Option<Vec<RawPathNode>> = None;
    loop {
        if let Some(nodes) = completed.take() {
            let parent = frames.last_mut().expect("a child level has a parent frame");
            let slot = parent
                .pending_slot
                .take()
                .expect("a completed level has a pending slot");
            parent.children[slot] = nodes;
            continue;
        }

        let frame = frames
            .last_mut()
            .expect("the root level remains until return");
        if !frame.flat_done {
            frame.flat_done = true;
            frame.paths = match materialize_flat_level(
                object,
                &record_context,
                frame.seeds,
                scale,
                context,
            ) {
                Ok(paths) => paths,
                Err(error) => {
                    consume_frames(frames);
                    return Err(error);
                }
            };
            frame.visits = visit_order(&frame.paths, frame.thin_walls, record);
            continue;
        }
        let Some(&slot) = frame.visits.get(frame.next_visit) else {
            let frame = frames.pop().expect("the root level remains until return");
            let nodes = frame
                .paths
                .into_iter()
                .zip(frame.children)
                .map(|(paths, children)| RawPathNode { paths, children })
                .collect();
            if frames.is_empty() {
                return Ok(PreparedRawPathSurface {
                    source_index: surface.source_index,
                    roots: nodes,
                    thin_walls: surface.thin_walls.clone(),
                });
            }
            completed = Some(nodes);
            continue;
        };
        frame.next_visit += 1;
        frame.pending_slot = Some(slot);
        let child_seeds = frame.seeds[slot].children.as_slice();
        frames.push(LevelFrame::new(child_seeds, None));
    }
}

/// The `traverse_loops` flat pass for one level — every same-level sibling
/// materialized (and fuzzified) in `loops` vector order.
fn materialize_flat_level(
    object: &PostClassicTraversalPrintObject,
    record_context: &super::RecordContext<'_>,
    seeds: &[TraversalSeed],
    scale: CoordinateScale,
    context: &mut crate::SlicerContext,
) -> Result<Vec<Vec<ExtrusionPath>>, SliceError> {
    let mut paths = Vec::with_capacity(seeds.len());
    for seed in seeds {
        match materialize_seed(
            object,
            super::RecordContext {
                record_index: record_context.record_index,
                record: record_context.record,
            },
            seed,
            scale,
            context,
        ) {
            Ok(paths_for_seed) => paths.push(paths_for_seed),
            Err(error) => {
                for level_paths in paths {
                    consume_paths(level_paths);
                }
                return Err(error);
            }
        }
    }
    Ok(paths)
}

/// Child-descent order of one materialized level — the chain the emission
/// traversal computes over the same loops (`PerimeterGenerator.cpp:238`,
/// mirrored by `entity_collections::traverse`), which also orders upstream's
/// `:254-255` child recursion. Slots compacted out of the chain (empty-path
/// loops, `PerimeterGenerator.cpp:208`) keep their subtrees materialized
/// afterwards in vector order.
fn visit_order(
    paths: &[Vec<ExtrusionPath>],
    thin_walls: Option<&[ThickPolyline]>,
    record: &ClassicTraversalRecord,
) -> Vec<usize> {
    let loop_count = paths.iter().filter(|paths| !paths.is_empty()).count();
    let overhang = matches!(record.branch, PendingPathBranch::OverhangClipping { .. });
    let thin_entities = match thin_walls {
        Some(thin_walls) if !thin_walls.is_empty() => convert_entities(
            thin_walls,
            record.ext_perimeter_flow,
            record.scale,
            ExtrusionRole::ExternalPerimeter,
        )
        .unwrap_or_default(),
        _ => Vec::new(),
    };
    let mut visits = if thin_entities.is_empty() {
        let first_points = paths
            .iter()
            .filter(|paths| !paths.is_empty())
            .map(|paths| {
                let [front, _] = loop_endpoints_after_reorder(paths, overhang);
                [front[0] as f64, front[1] as f64]
            })
            .collect::<Vec<_>>();
        chain_loops_by_first_points(&first_points)
    } else {
        let mut endpoints = Vec::with_capacity(loop_count + thin_entities.len());
        let mut is_loop = Vec::with_capacity(loop_count + thin_entities.len());
        for paths in paths.iter().filter(|paths| !paths.is_empty()) {
            endpoints.push(loop_endpoints_after_reorder(paths, overhang));
            is_loop.push(true);
        }
        for entity in &thin_entities {
            endpoints.push(loop_endpoints(entity));
            is_loop.push(matches!(entity, ExtrusionEntity::Loop(_)));
        }
        chain_extrusion_entities(&endpoints, &is_loop, [0, 0])
            .into_iter()
            .filter(|&(index, _)| index < loop_count)
            .map(|(index, _)| index)
            .collect()
    };
    visits.extend(loop_count..paths.len());
    visits
}

/// `[first, last]` endpoints of a materialized loop as the emission chain
/// reads them. `chained_loops::build_loop` reorders `OverhangClipping` paths
/// from the first point (`PerimeterGenerator.cpp:207`) before
/// `entity_collections` chains the loops, so replay that chain on a borrow to
/// read the post-reorder endpoints without mutating the stored paths.
fn loop_endpoints_after_reorder(paths: &[ExtrusionPath], overhang: bool) -> [[Coord; 2]; 2] {
    let front = paths[0].polyline.points[0];
    if !overhang {
        let back = paths
            .last()
            .and_then(|path| path.polyline.points.last())
            .copied()
            .unwrap_or(front);
        return [[front.x, front.y], [back.x, back.y]];
    }
    let chain = chain_extrusion_paths(paths, Some([front.x, front.y]));
    let front_entry = chain[0];
    let back_entry = *chain
        .last()
        .expect("a non-empty path set chains to a non-empty order");
    [
        chain_endpoint(paths, front_entry, false),
        chain_endpoint(paths, back_entry, true),
    ]
}

fn chain_endpoint(paths: &[ExtrusionPath], entry: (usize, bool), last: bool) -> [Coord; 2] {
    let points = &paths[entry.0].polyline.points;
    let point = if last != entry.1 {
        points[points.len() - 1]
    } else {
        points[0]
    };
    [point.x, point.y]
}

fn consume_frames(frames: Vec<LevelFrame<'_>>) {
    for frame in frames {
        for paths in frame.paths {
            consume_paths(paths);
        }
        for children in frame.children {
            consume_nodes(children);
        }
    }
}

fn consume_paths(paths: Vec<ExtrusionPath>) {
    for path in paths {
        let _ = (
            path.polyline.points,
            path.role,
            path.mm3_per_mm,
            path.width,
            path.height,
        );
    }
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
