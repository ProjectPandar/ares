//! Brim emission — the brim prints on object 0's first layer before the
//! object content (`GCode.cpp` emits it ahead of the layer's object
//! entries).

use super::super::{brim, motion, skirt};
use super::entry_builder::EntryBuilder;

#[expect(
    clippy::too_many_arguments,
    reason = "the brim branch shares the chunk loop's emit context"
)]
pub(super) fn emit(
    output: &mut Vec<u8>,
    state: &mut motion::EmitState,
    (brim, skirt): (&[Option<brim::BrimPlan>], &Option<skirt::SkirtPlan>),
    object_index: usize,
    copy: usize,
    layer_index: usize,
    entry_builder: &mut EntryBuilder<'_>,
    timelapse_context: super::super::timelapse::Context<'_>,
    chunk_perimeter_spacing: f64,
) -> Result<(), crate::SliceError> {
    if layer_index != 0 {
        return Ok(());
    }
    let grouped = skirt
        .as_ref()
        .and_then(|plan| plan.group_brims((object_index, copy)));
    // Group assembly (Print.cpp:4297-4389): the component owning this
    // instance emits at its first visiting chunk; emitting marks every
    // instance of the component done (m_objsWithBrim,
    // GCode.cpp:6056-6064,6073-6074).
    let plan = if let Some(component) = grouped.and_then(|components| {
        components
            .iter()
            .find(|component| component.matches((object_index, copy)) && !component.emitted(state))
    }) {
        component.mark_done(state);
        component.plan(brim)
    } else {
        if grouped.is_some() {
            return Ok(());
        }
        // No skirt groups own this instance's brim (no skirt plan):
        // the plan emits per copy (Brim.cpp:1004-1010,
        // GCode.cpp:6039-6082) with `ObjectInstanceID` dedup.
        let Some(plan) = brim[object_index].as_ref() else {
            return Ok(());
        };
        if plan.instances.iter().all(|instance| instance.copy != copy) {
            return Ok(());
        }
        if !state.emitted_brim_instances.insert((object_index, copy)) {
            return Ok(());
        }
        plan
    };
    let geometry = entry_builder.geometry(object_index, layer_index, chunk_perimeter_spacing);
    let lower_boundary = (!geometry.lower_boundary_lines.is_empty())
        .then(|| crate::geometry::LineDistanceTree::new(&geometry.lower_boundary_lines));
    // `GCode.cpp:4496/:4498-4500`: brim emission plans travels against
    // the external boundary; the window closes after the entities and
    // the one-shot disable is armed for the straight travel to the first
    // object point.
    state.use_external_mp = true;
    plan.emit(
        output,
        geometry.view(
            timelapse_context,
            object_index,
            layer_index,
            lower_boundary.as_ref(),
        ),
        state,
        copy,
    )?;
    state.use_external_mp = false;
    state.avoid_crossing_disabled_once = true;
    Ok(())
}
