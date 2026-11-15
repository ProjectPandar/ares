//! Support reversal carryover and print-object sharing.
//! Infill instead uses call-local copies (`GCode.cpp:8974–9008`).

use super::state::{EmitState, LayerGeometry};

/// The existing support ledger retains orientation state per PrintObject
/// and layer. Its ownership rewrite is separate from the infill port.
/// Raft layers are separate upstream `Layer` entity sets: their ledger keys
/// must not collide with the object layer keys of the same source object.
/// Upstream layer entity set a shared-reversal ledger is keyed by: object
/// layers, raft layers, and support layers are disjoint entity sets
/// (`PrintObject::layers()` / `m_support_layers`, Print.cpp:2103-2106).
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(in crate::project_slice::gcode_emit) enum SharedLayerId {
    Object(usize),
    Raft(usize),
    Support(usize),
}

/// The chained-emission scope of one `(print object, layer)` pass:
/// the emitting object (`session`, which discriminates objects sharing
/// one representative), the sharing-group representative source object,
/// and the shared layer entity set.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::project_slice::gcode_emit) struct ReversalScope {
    pub(in crate::project_slice::gcode_emit) session: usize,
    /// The representative print object `(source object, transform index)`.
    pub(in crate::project_slice::gcode_emit) representative: (usize, usize),
    pub(in crate::project_slice::gcode_emit) layer: SharedLayerId,
}

/// Emission-phase context: the layer geometry plus its shared-reversal
/// scope, threading the chained phases without widening signatures.
#[derive(Clone, Copy)]
pub(in crate::project_slice::gcode_emit) struct EmissionPhase<'a> {
    pub(in crate::project_slice::gcode_emit) geometry: LayerGeometry<'a>,
    pub(in crate::project_slice::gcode_emit) reversal: ReversalScope,
}

#[derive(Default)]
pub(in crate::project_slice::gcode_emit) struct SharedEntityReversals {
    support: std::collections::BTreeMap<((usize, usize), SharedLayerId), Vec<bool>>,
    /// `(last scope, cursor)` per ledger — the cursor resets whenever
    /// emission moves to another print object's layer pass (any of the
    /// session, representative, or layer changing) and otherwise advances
    /// over that pass's chained entities.
    support_cursor: (Option<ReversalScope>, usize),
}

impl SharedEntityReversals {
    fn chain<T: crate::project_slice::perimeters::classic::shortest_path::ChainEntity>(
        ledger: &mut std::collections::BTreeMap<((usize, usize), SharedLayerId), Vec<bool>>,
        cursor: &mut (Option<ReversalScope>, usize),
        scope: ReversalScope,
        entities: &mut Vec<T>,
        origin: (crate::geometry::Point, crate::geometry::CoordinateScale),
    ) {
        let (start, scale) = origin;
        let key = (scope.representative, scope.layer);
        if cursor.0 != Some(scope) {
            cursor.0 = Some(scope);
            cursor.1 = 0;
        }
        let base = cursor.1;
        let state = ledger.entry(key).or_default();
        if state.len() < base + entities.len() {
            state.resize(base + entities.len(), false);
        }
        let carried = state[base..base + entities.len()].to_vec();
        for (entity, flip) in entities.iter_mut().zip(carried) {
            if flip && !entity.is_loop() {
                entity.reverse();
            }
        }
        let flips =
            crate::project_slice::perimeters::classic::shortest_path::chain_and_reorder_entities_flips(
                entities,
                Some(start),
                scale,
            );
        for (slot, flip) in state[base..base + entities.len()].iter_mut().zip(flips) {
            if flip {
                *slot = !*slot;
            }
        }
        cursor.1 = base + entities.len();
    }

    pub(in crate::project_slice::gcode_emit) fn chain_support<
        T: crate::project_slice::perimeters::classic::shortest_path::ChainEntity,
    >(
        &mut self,
        scope: ReversalScope,
        entities: &mut Vec<T>,
        start: crate::geometry::Point,
        scale: crate::geometry::CoordinateScale,
    ) {
        Self::chain(
            &mut self.support,
            &mut self.support_cursor,
            scope,
            entities,
            (start, scale),
        );
    }
}

impl EmitState {
    /// Print.cpp:2193-2194,2240-2251: the first print object of an
    /// identical group slices; later ones share its layers — their
    /// chained-emission entity orientations observe the representative's
    /// reversals. Identity is per PRINT OBJECT: equal volume stacks
    /// (`shares_print_toolpaths`) AND an exactly equal PrintObject trafo
    /// (the post-shrinkage instance transform with the XY translation
    /// reset — `PrintApply.cpp:138-172`, upstream compares the full
    /// matrix with no tolerance), so instances of one source object with
    /// distinct rotations never share. Returns the representative
    /// `(source object, transform index)`, computed once per print object.
    pub(in crate::project_slice::gcode_emit) fn shared_toolpath_object(
        &mut self,
        traversal: &crate::project_slice::perimeters::classic::traversal::PreparedPostClassicTraversal,
        object_index: usize,
    ) -> (usize, usize) {
        let identity = traversal.objects[object_index]
            .predecessor
            .predecessor
            .predecessor
            .predecessor
            .object
            .identity();
        if let Some(&representative) = self.print_sharing.get(&identity) {
            return representative;
        }
        let resolved = |source: usize| {
            traversal
                .resolved
                .objects
                .iter()
                .find(|object| object.source_object_index == source)
                .expect("every sliced object has resolved config")
        };
        let objects = traversal.project.objects();
        let object = &objects[identity.0];
        let trafo = resolved(identity.0).print_objects[identity.1].transform;
        let representative = (0..=object_index)
            .map(|index| {
                let candidate = traversal.objects[index]
                    .predecessor
                    .predecessor
                    .predecessor
                    .predecessor
                    .object
                    .identity();
                (
                    candidate,
                    resolved(candidate.0).print_objects[candidate.1].transform,
                )
            })
            .find(|((source, _), candidate_trafo)| {
                objects[*source].shares_print_toolpaths(object) && *candidate_trafo == trafo
            })
            .expect("a print object shares toolpaths with itself")
            .0;
        self.print_sharing.insert(identity, representative);
        representative
    }
}

impl EmitState {
    /// The chained-emission scope for one `(print object, layer)` pass:
    /// `session` is the emitting ares print object (discriminating objects
    /// of one sharing group), the representative comes from
    /// `shared_toolpath_object`, and `layer` names the shared entity set.
    pub(in crate::project_slice::gcode_emit) fn reversal_scope(
        &mut self,
        traversal: &crate::project_slice::perimeters::classic::traversal::PreparedPostClassicTraversal,
        object_index: usize,
        layer: SharedLayerId,
    ) -> ReversalScope {
        ReversalScope {
            session: object_index,
            representative: self.shared_toolpath_object(traversal, object_index),
            layer,
        }
    }
}
