use crate::{
    SliceError,
    geometry::{Coord, CoordinateScale},
};

use super::{
    super::{
        entity_collections::ExtrusionEntity,
        gap_extrusion::variable_width::convert_entities,
        shortest_path::{
            chain_and_reorder_extrusion_paths, chain_extrusion_entities,
            chain_loops_by_first_points,
        },
        traversal::{
            PendingPathBranch, PostClassicTraversalPrintObject, PreparedTraversalSurface,
            TraversalSeed,
        },
    },
    path::materialize_seed,
    types::{ExtrusionPath, ExtrusionRole, PreparedRawPathSurface, RawPathNode},
};

/// A finished child level returned to its parent frame: the wrapped nodes
/// plus the chain computed for that level (`PerimeterGenerator.cpp:238`).
struct RebuiltLevel {
    nodes: Vec<RawPathNode>,
    chain: Vec<(usize, bool)>,
}

/// One `traverse_loops` recursion level (`PerimeterGenerator.cpp:142-151`
/// flat pass + `:254-255` child descent). The upstream flat pass fuzzifies
/// every same-level sibling in `loops` order before any child recursion, so
/// the shared seeded stream is consumed sibling-first while the emitted
/// `RawPathNode` tree keeps the children-first structure the emission
/// traversal (`:274-278`) expects. The level chain (`:238`) is computed here
/// once and stored on the tree for the emission traversal to consume.
struct LevelFrame<'a> {
    seeds: &'a [TraversalSeed],
    /// Thin-wall entities chain with the root level only
    /// (`PerimeterGenerator.cpp:230-233`); child levels pass `None`.
    thin_entities: Option<&'a [ExtrusionEntity]>,
    flat_done: bool,
    paths: Vec<Vec<ExtrusionPath>>,
    /// Child visit order: chain-ordered loop slots, then the compacted-out
    /// tail slots in vector order.
    visits: Vec<usize>,
    next_visit: usize,
    pending_slot: Option<usize>,
    children: Vec<Vec<RawPathNode>>,
    /// Each slot's children-level chain, captured on completion for the
    /// assembled node's `child_order`.
    chains: Vec<Vec<(usize, bool)>>,
    /// This level's own chain, held from the flat pass until assembly.
    chain: Vec<(usize, bool)>,
}

impl<'a> LevelFrame<'a> {
    fn new(seeds: &'a [TraversalSeed], thin_entities: Option<&'a [ExtrusionEntity]>) -> Self {
        Self {
            seeds,
            thin_entities,
            flat_done: false,
            paths: Vec::with_capacity(seeds.len()),
            visits: Vec::new(),
            next_visit: 0,
            pending_slot: None,
            children: std::iter::repeat_with(Vec::new).take(seeds.len()).collect(),
            chains: std::iter::repeat_with(Vec::new).take(seeds.len()).collect(),
            chain: Vec::new(),
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
    // `variable_width(thin_walls, erExternalPerimeter, ext_perimeter_flow,
    // coll.entities)` (`PerimeterGenerator.cpp:231`) — materialized once here
    // because the root chain interleaves the entities; the emission traversal
    // consumes them verbatim.
    let thin_entities = convert_entities(
        &surface.thin_walls,
        record.ext_perimeter_flow,
        record.scale,
        ExtrusionRole::ExternalPerimeter,
    )
    .unwrap_or_default();
    let mut frames = vec![LevelFrame::new(&surface.roots, Some(&thin_entities))];
    let mut completed: Option<RebuiltLevel> = None;
    loop {
        if let Some(rebuilt) = completed.take() {
            let parent = frames.last_mut().expect("a child level has a parent frame");
            let slot = parent
                .pending_slot
                .take()
                .expect("a completed level has a pending slot");
            parent.children[slot] = rebuilt.nodes;
            parent.chains[slot] = rebuilt.chain;
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
            let chain = level_chain(&frame.paths, frame.thin_entities);
            let loop_count = frame.paths.iter().filter(|paths| !paths.is_empty()).count();
            frame.visits = chain
                .iter()
                .filter(|&&(index, _)| index < loop_count)
                .map(|&(index, _)| index)
                .collect();
            frame.visits.extend(loop_count..frame.paths.len());
            frame.chain = chain;
            continue;
        }
        let Some(&slot) = frame.visits.get(frame.next_visit) else {
            let mut frame = frames.pop().expect("the root level remains until return");
            let chain = std::mem::take(&mut frame.chain);
            let nodes = frame
                .paths
                .into_iter()
                .zip(frame.children)
                .zip(frame.chains)
                .map(|((paths, children), child_order)| RawPathNode {
                    paths,
                    children,
                    child_order,
                })
                .collect();
            if frames.is_empty() {
                return Ok(PreparedRawPathSurface {
                    source_index: surface.source_index,
                    roots: nodes,
                    root_order: chain,
                    thin_entities,
                });
            }
            completed = Some(RebuiltLevel { nodes, chain });
            continue;
        };
        frame.next_visit += 1;
        frame.pending_slot = Some(slot);
        let child_seeds = frame.seeds[slot].children.as_slice();
        frames.push(LevelFrame::new(child_seeds, None));
    }
}

/// The `traverse_loops` flat pass for one level — every same-level sibling
/// materialized (and fuzzified) in `loops` vector order. The overhang branch
/// re-chains its paths from the first point (`PerimeterGenerator.cpp:207`)
/// here in the flat pass, so the stored paths and the level chain read the
/// same final path order.
fn materialize_flat_level(
    object: &PostClassicTraversalPrintObject,
    record_context: &super::RecordContext<'_>,
    seeds: &[TraversalSeed],
    scale: CoordinateScale,
    context: &mut crate::SlicerContext,
) -> Result<Vec<Vec<ExtrusionPath>>, SliceError> {
    let overhang = matches!(
        record_context.record.branch,
        PendingPathBranch::OverhangClipping { .. }
    );
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
            Ok(mut paths_for_seed) => {
                if overhang && !paths_for_seed.is_empty() {
                    let first = paths_for_seed[0].polyline.points[0];
                    chain_and_reorder_extrusion_paths(&mut paths_for_seed, [first.x, first.y]);
                }
                paths.push(paths_for_seed);
            }
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

/// The level chain (`PerimeterGenerator.cpp:238`) over the compacted
/// non-empty loops, mixed with the root level's thin-wall entities. Loop
/// entries carry `reverse == false` (loop chain semantics); thin entries may
/// reverse. Indices are compacted loop slots, and the emission traversal maps
/// them onto the uncompacted seeds with the upstream slot-sliding quirk.
fn level_chain(
    paths: &[Vec<ExtrusionPath>],
    thin_entities: Option<&[ExtrusionEntity]>,
) -> Vec<(usize, bool)> {
    let loop_count = paths.iter().filter(|paths| !paths.is_empty()).count();
    match thin_entities {
        None | Some([]) => {
            let first_points = paths
                .iter()
                .filter(|paths| !paths.is_empty())
                .map(|paths| {
                    let first = paths[0].polyline.points[0];
                    [first.x as f64, first.y as f64]
                })
                .collect::<Vec<_>>();
            chain_loops_by_first_points(&first_points)
                .into_iter()
                .map(|index| (index, false))
                .collect()
        }
        Some(thin_entities) => {
            let mut endpoints = Vec::with_capacity(loop_count + thin_entities.len());
            let mut is_loop = Vec::with_capacity(loop_count + thin_entities.len());
            for paths in paths.iter().filter(|paths| !paths.is_empty()) {
                let first = paths[0].polyline.points[0];
                let last = paths
                    .last()
                    .and_then(|path| path.polyline.points.last())
                    .copied()
                    .unwrap_or(first);
                endpoints.push([[first.x, first.y], [last.x, last.y]]);
                is_loop.push(true);
            }
            for entity in thin_entities {
                endpoints.push(entity_endpoints(entity));
                is_loop.push(matches!(entity, ExtrusionEntity::Loop(_)));
            }
            chain_extrusion_entities(&endpoints, &is_loop, [0, 0])
        }
    }
}

fn entity_endpoints(entity: &ExtrusionEntity) -> [[Coord; 2]; 2] {
    match entity {
        ExtrusionEntity::Loop(ordered) => {
            let first = ordered.extrusion_loop.paths[0].polyline.points[0];
            let last = ordered
                .extrusion_loop
                .paths
                .last()
                .and_then(|p| p.polyline.points.last())
                .copied()
                .unwrap_or(first);
            [[first.x, first.y], [last.x, last.y]]
        }
        ExtrusionEntity::Path(path) => {
            let first = path.polyline.points[0];
            let last = path.polyline.points.last().copied().unwrap_or(first);
            [[first.x, first.y], [last.x, last.y]]
        }
        ExtrusionEntity::MultiPath(multi) => {
            let first = multi.paths[0].polyline.points[0];
            let last = multi
                .paths
                .last()
                .and_then(|p| p.polyline.points.last())
                .copied()
                .unwrap_or(first);
            [[first.x, first.y], [last.x, last.y]]
        }
    }
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
