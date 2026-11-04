//! `OozePrevention` (`GCode.cpp:267–309,2280–2283`).

use super::{change::Context, motion::EmitState};

pub(super) fn pre(output: &mut Vec<u8>, state: &EmitState, context: Context<'_>) {
    let full = &context.traversal.resolved.views.full;
    if !full.process.print.ooze_prevention.0 || state.options.single_extruder_multi_material {
        return;
    }
    let id = state.extruders.active_id();
    let is_last_use = context
        .tool_ordering
        .is_some_and(|ordering| ordering.is_last_extrusion_layer(context.print_z, id));
    if is_last_use {
        super::super::temperature::append(
            output,
            context.traversal.resolved.views.runtime_gcode.gcode_flavor,
            0,
            false,
            Some(id),
        );
        output.pop();
        output.extend_from_slice(b" ;cooldown\n");
        return;
    }
    let idle = at(&full.filament.print.idle_temperature, id);
    let delta = full.process.print.standby_temperature_delta.0;
    let temperature = if idle != 0 {
        idle
    } else if delta != 0 {
        operating(state, context).wrapping_add(delta)
    } else {
        return;
    };
    super::super::temperature::append(
        output,
        context.traversal.resolved.views.runtime_gcode.gcode_flavor,
        temperature as u32,
        false,
        Some(id),
    );
    output.pop();
    output.extend_from_slice(b" ;cooldown\n");
}

pub(super) fn post(output: &mut Vec<u8>, state: &EmitState, context: Context<'_>) {
    let print = &context.traversal.resolved.views.full.process.print;
    if print.ooze_prevention.0
        && !state.options.single_extruder_multi_material
        && print.standby_temperature_delta.0 != 0
    {
        super::super::temperature::append(
            output,
            context.traversal.resolved.views.runtime_gcode.gcode_flavor,
            operating(state, context) as u32,
            true,
            Some(state.extruders.active_id()),
        );
    }
}

fn operating(state: &EmitState, context: Context<'_>) -> i32 {
    let filament = &context.traversal.resolved.views.full.filament.print;
    let id = state.extruders.active_id();
    let ordinary = at(&filament.nozzle_temperature, id);
    if context.layer_num < 0 || state.layer_index == 0 || ordinary == 0 {
        at(&filament.nozzle_temperature_initial_layer, id)
    } else {
        ordinary
    }
}

fn at(values: &crate::OrcaInts, id: usize) -> i32 {
    values.0.get(id).unwrap_or(&values.0[0]).0
}
