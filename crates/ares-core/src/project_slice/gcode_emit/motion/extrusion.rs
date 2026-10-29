#[cfg(test)]
mod tests;

use super::{
    EmitState, arc,
    features::{PathProperties, feature_description, is_bridge_feature},
    format::{axis as format_axis, extrusion as format_extrusion, z as format_z},
};

pub(super) struct XyzEmission<'a> {
    pub(super) end: arc::Point,
    pub(super) z: f64,
    pub(super) delta: f64,
    pub(super) properties: PathProperties<'a>,
}

/// `GCodeWriter.cpp:975-1000::extrude_to_xyz`, with extrusion enabled.
pub(super) fn extrude_to_xyz(
    output: &mut Vec<u8>,
    command: XyzEmission<'_>,
    state: &mut EmitState,
) {
    let XyzEmission {
        end,
        z,
        delta,
        properties,
    } = command;
    // `GCodeWriter.hpp:226-228::quantize_xyzf` and writer line 982.
    let quantize = |value: f64| (value * 1000.0).round() * 0.001;
    let z_changed = state
        .writer_z
        .is_none_or(|previous| quantize(z) != quantize(previous));
    let z_word = if z_changed {
        format!(" Z{}", format_z(z))
    } else {
        String::new()
    };
    let value = coordinate(state, delta);
    let comment = if state.options.gcode_comments {
        let bridge = if is_bridge_feature(properties.feature) {
            " (bridge)"
        } else {
            ""
        };
        format!(" ; {}{bridge}", feature_description(properties.feature))
    } else {
        String::new()
    };
    output.extend_from_slice(
        format!(
            "G1 X{} Y{}{} E{}{comment}\n",
            format_axis(end.x),
            format_axis(end.y),
            z_word,
            format_extrusion(value),
        )
        .as_bytes(),
    );
    state.x = end.x;
    state.y = end.y;
    state.scarf_z = Some(z);
    // `m_pos = point`: raw position advances even with an omitted Z word.
    state.record_z(z);
    state.wipe_start = Some(end);
}

pub(super) fn force_no_extrusion_segment(
    output: &mut Vec<u8>,
    end: arc::Point,
    state: &mut EmitState,
) {
    let comment = if state.options.gcode_comments {
        " ; move inwards before retraction/seam"
    } else {
        ""
    };
    output.extend_from_slice(
        format!(
            "G1 X{} Y{}{comment}\n",
            format_axis(end.x),
            format_axis(end.y)
        )
        .as_bytes(),
    );
    state.x = end.x;
    state.y = end.y;
    state.wipe_start = Some(end);
}

pub(super) fn linear_segment(
    output: &mut Vec<u8>,
    end: arc::Point,
    length: f64,
    properties: PathProperties<'_>,
    state: &mut EmitState,
) {
    let extrusion = for_length(
        length,
        effective_flow_at_emission(properties, state),
        state.options.filament_flow_ratio,
        state.options.filament_area,
    ) * small_area_multiplier(state, properties.feature, length);
    let comment = if state.options.gcode_comments {
        let bridge = if is_bridge_feature(properties.feature) {
            " (bridge)"
        } else {
            ""
        };
        format!(" ; {}{bridge}", feature_description(properties.feature))
    } else {
        Default::default()
    };
    if extrusion.abs() <= f64::EPSILON {
        output.extend_from_slice(
            format!(
                "G1 X{} Y{}{comment}\n",
                format_axis(end.x),
                format_axis(end.y)
            )
            .as_bytes(),
        );
    } else {
        let extrusion = coordinate(state, extrusion);
        output.extend_from_slice(
            format!(
                "G1 X{} Y{} E{}{comment}\n",
                format_axis(end.x),
                format_axis(end.y),
                format_extrusion(extrusion)
            )
            .as_bytes(),
        );
    }
    state.x = end.x;
    state.y = end.y;
    state.wipe_start = Some(end);
}

pub(super) fn small_area_multiplier(state: &EmitState, feature: &str, length: f64) -> f64 {
    if !state.small_area_gate.needs(feature, state.on_first_layer) {
        return 1.0;
    }
    state
        .small_area_flow
        .multiplier_for_feature(feature, state.on_first_layer, length)
}

pub(super) fn coordinate(state: &mut EmitState, delta: f64) -> f64 {
    // `Extruder::extrude`: relative E resets the axis, not absolute usage.
    if state.options.use_relative_e_distances {
        state.extruder.e_position = 0.0;
    }
    state.extruder.e_position += delta;
    // Mirror Orca's `Extruder::extrude()`: `m_absolute_E += dE` for any
    // sign, and negative `dE` accumulates into `m_retracted`. With
    // `single_extruder_multi_material` (share extruder) `used_filament()`
    // returns the signed `m_absolute_E`; otherwise `m_absolute_E +
    // m_retracted`. Template output never calls this.
    state.extruder.filament_used += delta;
    if delta < 0.0 {
        state.extruder.retracted_amount -= delta;
    }
    if state.options.use_relative_e_distances {
        delta
    } else {
        state.extruder.e_position
    }
}

/// `GCode.cpp:6469-6507`: one effective flow for extrusion, speed caps and PA.
/// Earlier ungated fill ratios still arrive in the geometric handoff.
pub(super) fn effective_flow(
    properties: PathProperties<'_>,
    options: &super::MotionOptions,
    on_first_layer: bool,
) -> f64 {
    let mut flow = properties.mm3_per_mm * options.print_flow_ratio;
    flow *= options.filament_flow_ratio;
    if let Some(slope) = properties.slope {
        flow *= slope.flow_ratio;
    }
    if options.set_other_flow_ratios {
        flow *= match properties.feature {
            "Outer wall" => options.outer_wall_flow_ratio,
            "Inner wall" => options.inner_wall_flow_ratio,
            "Overhang wall" => options.overhang_flow_ratio,
            "Sparse infill" => options.sparse_infill_flow_ratio,
            "Internal solid infill" => options.internal_solid_infill_flow_ratio,
            "Gap infill" => options.gap_fill_flow_ratio,
            "Support" => options.support_flow_ratio,
            "Support interface" => options.support_interface_flow_ratio,
            _ => 1.0,
        };
        if on_first_layer && !matches!(properties.feature, "Brim" | "Skirt") {
            flow *= options.first_layer_flow_ratio;
        }
    }
    flow
}

/// GCode.cpp:6479-6480 applies the live object ratio after print/filament flow.
pub(super) fn effective_flow_at_emission(properties: PathProperties<'_>, state: &EmitState) -> f64 {
    let flow = effective_flow(properties, &state.options, state.on_first_layer);
    if properties.feature == "Brim" {
        flow * state.current_brim_flow_ratio
    } else {
        flow
    }
}

pub(super) fn for_length(
    length: f64,
    effective_mm3_per_mm: f64,
    filament_flow_ratio: f64,
    filament_area: f64,
) -> f64 {
    let mut e_per_mm3 = filament_flow_ratio;
    e_per_mm3 /= filament_area;
    let mut e_per_mm = e_per_mm3 * effective_mm3_per_mm;
    e_per_mm /= filament_flow_ratio;
    e_per_mm * length
}

pub(super) fn speed(output: &mut Vec<u8>, feedrate: f64, properties: PathProperties<'_>) {
    let external = if properties.feature == "Outer wall" {
        ";_EXTERNAL_PERIMETER"
    } else {
        ""
    };
    output.extend_from_slice(
        format!(
            "G1 F{};_EXTRUDE_SET_SPEED{external}\n",
            format_axis(feedrate)
        )
        .as_bytes(),
    );
}
