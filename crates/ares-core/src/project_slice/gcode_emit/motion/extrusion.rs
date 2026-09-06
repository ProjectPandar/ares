#[cfg(test)]
mod tests;

use super::{
    EmitState, arc,
    features::PathProperties,
    format::{axis as format_axis, extrusion as format_extrusion},
};

pub(super) fn linear_segment(
    output: &mut Vec<u8>,
    end: arc::Point,
    length: f64,
    properties: PathProperties<'_>,
    state: &mut EmitState,
) {
    let extrusion = for_length(
        length,
        properties.mm3_per_mm,
        state.options.filament_flow_ratio,
        state.options.print_flow_ratio,
        state.options.filament_area,
    ) * state
        .small_area_flow
        .multiplier_for_feature(properties.feature, length);
    if extrusion.abs() <= f64::EPSILON {
        output.extend_from_slice(
            format!("G1 X{} Y{}\n", format_axis(end.x), format_axis(end.y)).as_bytes(),
        );
    } else {
        let extrusion = coordinate(state, extrusion);
        output.extend_from_slice(
            format!(
                "G1 X{} Y{} E{}\n",
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

pub(super) fn coordinate(state: &mut EmitState, delta: f64) -> f64 {
    state.e_position += delta;
    // Mirror Orca's `Extruder::extrude()`: `m_absolute_E += dE` for any
    // sign, and negative `dE` accumulates into `m_retracted`. With
    // `single_extruder_multi_material` (share extruder) `used_filament()`
    // returns the signed `m_absolute_E`; otherwise `m_absolute_E +
    // m_retracted`. Template output never calls this.
    state.filament_used += delta;
    if delta < 0.0 {
        state.retracted_amount -= delta;
    }
    if state.options.use_relative_e_distances {
        delta
    } else {
        state.e_position
    }
}

pub(super) fn for_length(
    length: f64,
    mm3_per_mm: f64,
    filament_flow_ratio: f64,
    print_flow_ratio: f64,
    filament_area: f64,
) -> f64 {
    let mut effective_mm3_per_mm = mm3_per_mm * print_flow_ratio;
    effective_mm3_per_mm *= filament_flow_ratio;
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
