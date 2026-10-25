use crate::ProcessWallDirection;

use super::{
    super::chained_loops::{ChainedLoopNode, ExtrusionLoop},
    super::traversal::TraversalSeed,
    orientation::orient_loop,
    types::{ExtrusionEntity, ExtrusionEntityCollection, OrderedExtrusionLoop},
};

struct PendingParent {
    extrusion_loop: ExtrusionLoop,
    is_contour: bool,
    depth: u16,
}

struct GroupFrame<'a> {
    nodes: Vec<ChainedLoopNode>,
    seeds: &'a [TraversalSeed],
    /// Number of materialized loop entities = the thin-entity boundary
    /// (`entities[loop_count..]` are thin walls).
    ///
    /// `entities` is compacted (a node whose `extrusion_loop` is `None` — an
    /// empty-overhang root, materialization flat pass `PerimeterGenerator.cpp:208`
    /// `continue` — contributes no entity), but `seeds`/`nodes` are NOT
    /// compacted: loop entities pair with `seeds[index]`/`nodes[index]` at the
    /// same slot index, sliding left so a `None` root's children still descend
    /// through the slot the next real loop occupies
    /// (`preserves_source_compact_entity_against_original_loop_indexing`).
    /// Upstream has the same compaction: `coll.entities` compacts while `loops`
    /// does not — upstream's `idx.first >= loops.size()` thin boundary (:241)
    /// then reads the uncompacted count. `loop_count` is the equivalent
    /// boundary over ares's compacted `entities`; the two agree in every
    /// non-degenerate case and diverge only when a thin entity lands inside
    /// `loops.size()`: an open thin wall is an `ExtrusionPath*`, so upstream
    /// `static_cast<ExtrusionLoop*>` (:257) is a bad downcast (UB), while a
    /// closed thin wall materializes as a real `ExtrusionLoop`
    /// (`gap_extrusion/variable_width.rs:43-49`) and upstream silently
    /// assigns the shifted loop's seed/depth/children — wrong topology rather
    /// than UB. `loop_count` avoids both.
    loop_count: usize,
    /// Loop entities occupy `entities[..loop_count]`; thin-wall entities (only
    /// ever present on the root frame) occupy `entities[loop_count..]`
    /// (upstream `coll.entities` ordering — loops first, thin walls appended,
    /// `PerimeterGenerator.cpp:230-238`).
    entities: Vec<Option<ExtrusionEntity>>,
    /// The level chain computed once at materialization
    /// (`PerimeterGenerator.cpp:238`) and carried through
    /// `RawPathNode::child_order` / the surface `root_order`. Consumed here
    /// verbatim — descent order and emission order are the same chain, never
    /// recomputed.
    chain: Vec<(usize, bool)>,
    next: usize,
    reverse_thin_wall_hole: bool,
    pending_parent: Option<PendingParent>,
    output: ExtrusionEntityCollection,
}

impl<'a> GroupFrame<'a> {
    fn new(
        mut nodes: Vec<ChainedLoopNode>,
        seeds: &'a [TraversalSeed],
        reverse_thin_wall_hole: bool,
        thin_entities: Vec<ExtrusionEntity>,
        chain: Vec<(usize, bool)>,
    ) -> Self {
        assert_eq!(nodes.len(), seeds.len());
        // Compact out `None` loops (empty-overhang nodes) into `entities`; the
        // surviving loop count is the thin-entity boundary, matching the
        // compaction the materialization chain was computed over.
        let loops: Vec<ExtrusionLoop> = nodes
            .iter_mut()
            .filter_map(|node| node.extrusion_loop.take())
            .collect();
        let loop_count = loops.len();
        let mut entities: Vec<Option<ExtrusionEntity>> = loops
            .into_iter()
            .map(|extrusion_loop| {
                Some(ExtrusionEntity::Loop(OrderedExtrusionLoop {
                    unsupported_distance: None,
                    extrusion_loop,
                    inset_idx: 0, // overwritten by the chain order below
                }))
            })
            .collect();
        // Append thin-wall entities (PerimeterGenerator.cpp:231-238) —
        // materialized once at materialization, only ever on the root frame.
        entities.extend(thin_entities.into_iter().map(Some));
        Self {
            nodes,
            seeds,
            loop_count,
            entities,
            chain,
            next: 0,
            reverse_thin_wall_hole,
            pending_parent: None,
            output: ExtrusionEntityCollection::default(),
        }
    }

    /// `begin_next` — returns the loop entity to descend into as
    /// `Ok((children, child_seeds, child_order, rev))`, or `Err(entity,
    /// reverse)` for a thin-wall entity emitted directly (upstream
    /// `idx.first >= loops.size()` branch, `PerimeterGenerator.cpp:241-247`).
    fn begin_next(&mut self) -> Option<Result<LoopDescent<'a>, (ExtrusionEntity, bool)>> {
        let &(index, reverse) = self.chain.get(self.next)?;
        self.next += 1;
        let entity = self.entities[index]
            .take()
            .expect("entity chain indices are unique");
        if index >= self.loop_count {
            // Thin-wall entity: emit directly, honoring chain reversal.
            return Some(Err((entity, reverse)));
        }
        let ExtrusionEntity::Loop(ordered) = entity else {
            unreachable!("loop slots hold loop entities");
        };
        let extrusion_loop = ordered.extrusion_loop;
        let seed = &self.seeds[index];
        let reverse_children_thin_wall_hole = self.seeds.len() == 1
            && seed.is_contour
            && seed.children.len() == 1
            && !seed.children[0].is_contour
            && seed.children[0].children.is_empty();
        let node = &mut self.nodes[index];
        let children = std::mem::take(&mut node.children);
        let child_order = std::mem::take(&mut node.child_order);
        self.pending_parent = Some(PendingParent {
            extrusion_loop,
            is_contour: seed.is_contour,
            depth: seed.depth,
        });
        Some(Ok((
            children,
            seed.children.as_slice(),
            child_order,
            reverse_children_thin_wall_hole,
        )))
    }
}

/// Children, child seeds, their precomputed chain order, and chain reversal of
/// a loop entity to descend into.
type LoopDescent<'a> = (
    Vec<ChainedLoopNode>,
    &'a [TraversalSeed],
    Vec<(usize, bool)>,
    bool,
);

pub(super) fn traverse_loops(
    roots: Vec<ChainedLoopNode>,
    seeds: &[TraversalSeed],
    wall_direction: ProcessWallDirection,
    thin_entities: Vec<ExtrusionEntity>,
    root_order: Vec<(usize, bool)>,
) -> ExtrusionEntityCollection {
    let mut frames = vec![GroupFrame::new(
        roots,
        seeds,
        false,
        thin_entities,
        root_order,
    )];
    let mut completed: Option<ExtrusionEntityCollection> = None;
    loop {
        if let Some(mut children) = completed.take() {
            let frame = frames.last_mut().expect("a child has a parent frame");
            let mut parent = frame
                .pending_parent
                .take()
                .expect("a child result has a pending parent");
            orient_loop(
                &mut parent.extrusion_loop,
                wall_direction,
                parent.is_contour,
                frame.reverse_thin_wall_hole,
            );
            if frame.reverse_thin_wall_hole {
                frame.output.entities.reverse();
            }
            let is_contour = parent.is_contour;
            let parent = ExtrusionEntity::Loop(OrderedExtrusionLoop {
                unsupported_distance: None,
                extrusion_loop: parent.extrusion_loop,
                inset_idx: i32::from(parent.depth),
            });
            if is_contour {
                frame.output.entities.append(&mut children.entities);
                frame.output.entities.push(parent);
            } else {
                frame.output.entities.push(parent);
                frame.output.entities.append(&mut children.entities);
            }
            continue;
        }

        match frames
            .last_mut()
            .expect("the root frame remains until return")
            .begin_next()
        {
            Some(Ok((children, child_seeds, child_order, reverse_children_thin_wall_hole))) => {
                frames.push(GroupFrame::new(
                    children,
                    child_seeds,
                    reverse_children_thin_wall_hole,
                    Vec::new(), // thin walls only on the root frame
                    child_order,
                ));
            }
            Some(Err((entity, reverse))) => {
                // Thin-wall entity emitted in chain position (upstream :241-247);
                // `idx.second` reverses it when the chain walked it backwards.
                let frame = frames.last_mut().expect("the root frame remains");
                let mut entity = entity;
                if reverse {
                    match &mut entity {
                        ExtrusionEntity::Loop(o) => o.extrusion_loop.reverse(),
                        ExtrusionEntity::Path(p) => p.reverse(),
                        ExtrusionEntity::MultiPath(m) => m.paths.reverse(),
                    }
                }
                frame.output.entities.push(entity);
            }
            None => {
                let frame = frames.pop().expect("a frame exists");
                drain_nodes(frame.nodes);
                let result = frame.output;
                if frames.is_empty() {
                    return result;
                }
                completed = Some(result);
            }
        }
    }
}

fn drain_nodes(mut nodes: Vec<ChainedLoopNode>) {
    // Iterative work-list — a deep loop chain must not recurse (the constrained
    // stack test builds 2_000-deep nesting).
    while let Some(mut node) = nodes.pop() {
        nodes.append(&mut node.children);
    }
}
