//! Brim emission — the brim prints on object 0's first layer before the
//! object content (`GCode.cpp` emits it ahead of the layer's object
//! entries).

use super::super::{brim, motion, skirt};
use super::entry_builder::EntryBuilder;
use crate::project_slice::perimeters::classic::traversal::types::PreparedPostClassicTraversal;

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
    traversal: &PreparedPostClassicTraversal,
    chunk_perimeter_spacing: f64,
) -> Result<(), crate::SliceError> {
    if layer_index != 0 {
        return Ok(());
    }
    let grouped = skirt
        .as_ref()
        .and_then(|plan| plan.group_brims((object_index, copy)));
    let (plan, owners) = if let Some(components) = grouped {
        let Some(component) = components.iter().find(|component| {
            component.owners().contains(&object_index)
                && component
                    .owners()
                    .iter()
                    .any(|owner| !state.emitted_brim_owners.contains(owner))
        }) else {
            return Ok(());
        };
        (component.plan(brim), component.owners())
    } else {
        let global = brim.iter().flatten().find(|plan| plan.global);
        let plan = if let Some(plan) = global {
            (!state.global_brim_done).then_some(plan)
        } else {
            brim[object_index].as_ref().filter(|plan| {
                !plan.owner_collection || !state.emitted_brim_owners.contains(&object_index)
            })
        };
        let Some(plan) = plan else {
            return Ok(());
        };
        (plan, std::slice::from_ref(&object_index))
    };
    if plan.global {
        state.global_brim_done = true;
    }
    // Brim.cpp:1004-1010 / GCode.cpp:6039-6082: uncombined per-instance
    // brims dedup by `ObjectInstanceID` (`m_objsWithBrim`); each copy emits
    // its own plan at its own chunk. Merged (combined) plans keep the
    // owner-level dedup.
    let per_instance = plan.owner_collection && !plan.instances.is_empty();
    if per_instance {
        if plan.instances.iter().all(|instance| instance.copy != copy) {
            return Ok(());
        }
        if !state.emitted_brim_instances.insert((object_index, copy)) {
            return Ok(());
        }
    } else if plan.owner_collection {
        state.emitted_brim_owners.extend(owners.iter().copied());
    }
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
            traversal,
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
