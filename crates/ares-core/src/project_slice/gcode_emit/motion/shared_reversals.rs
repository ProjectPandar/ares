//! Shared-entity reversal carryover for chained emission
//! (`GCode.cpp:6164,6201` / `ShortestPath.cpp:1040-1052`).

use super::state::{EmitState, LayerGeometry};

/// `chain_and_reorder_extrusion_entities` runs over the pointer vector into
/// the PrintObject's layer entities and reverses chosen entities IN PLACE
/// (`ShortestPath.cpp:1040-1052`), so instances emitted later in the same
/// layer observe the mutated orientations. Ares re-runs fill per instance,
/// so the orientation state is carried explicitly: keyed by
/// `(source_object_index, layer id)` with a positional entity index inside
/// the per-`(object, layer)` chained-emission sequence (structurally
/// identical across instances of one source object).
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
    pub(in crate::project_slice::gcode_emit) representative: usize,
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
    infill: std::collections::BTreeMap<(usize, SharedLayerId), Vec<bool>>,
    support: std::collections::BTreeMap<(usize, SharedLayerId), Vec<bool>>,
    /// `(last scope, cursor)` per ledger — the cursor resets whenever
    /// emission moves to another print object's layer pass (any of the
    /// session, representative, or layer changing) and otherwise advances
    /// over that pass's chained entities.
    infill_cursor: (Option<ReversalScope>, usize),
    support_cursor: (Option<ReversalScope>, usize),
}

impl SharedEntityReversals {
    fn chain<T: crate::project_slice::perimeters::classic::shortest_path::ChainEntity>(
        ledger: &mut std::collections::BTreeMap<(usize, SharedLayerId), Vec<bool>>,
        cursor: &mut (Option<ReversalScope>, usize),
        scope: ReversalScope,
        entities: &mut Vec<T>,
        start: crate::geometry::Point,
    ) {
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
            );
        for (slot, flip) in state[base..base + entities.len()].iter_mut().zip(flips) {
            if flip {
                *slot = !*slot;
            }
        }
        cursor.1 = base + entities.len();
    }

    pub(in crate::project_slice::gcode_emit) fn chain_infill<
        T: crate::project_slice::perimeters::classic::shortest_path::ChainEntity,
    >(
        &mut self,
        scope: ReversalScope,
        entities: &mut Vec<T>,
        start: crate::geometry::Point,
    ) {
        Self::chain(
            &mut self.infill,
            &mut self.infill_cursor,
            scope,
            entities,
            start,
        );
    }

    pub(in crate::project_slice::gcode_emit) fn chain_support<
        T: crate::project_slice::perimeters::classic::shortest_path::ChainEntity,
    >(
        &mut self,
        scope: ReversalScope,
        entities: &mut Vec<T>,
        start: crate::geometry::Point,
    ) {
        Self::chain(
            &mut self.support,
            &mut self.support_cursor,
            scope,
            entities,
            start,
        );
    }
}

impl EmitState {
    /// Print.cpp:2240-2251: the first print object of an identical group
    /// slices; later ones share its layers — their chained-emission entity
    /// orientations observe the representative's reversals. Returns the
    /// representative source object index, computed once per object.
    pub(in crate::project_slice::gcode_emit) fn shared_toolpath_object(
        &mut self,
        project: &crate::project::Project,
        source_object_index: usize,
    ) -> usize {
        if let Some(&representative) = self.print_sharing.get(&source_object_index) {
            return representative;
        }
        let objects = project.objects();
        let object = &objects[source_object_index];
        let representative = objects
            .iter()
            .take(source_object_index + 1)
            .position(|candidate| candidate.shares_print_toolpaths(object))
            .expect("an object shares toolpaths with itself");
        self.print_sharing
            .insert(source_object_index, representative);
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
        project: &crate::project::Project,
        session: usize,
        source_object_index: usize,
        layer: SharedLayerId,
    ) -> ReversalScope {
        ReversalScope {
            session,
            representative: self.shared_toolpath_object(project, source_object_index),
            layer,
        }
    }
}
