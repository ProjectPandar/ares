use super::Slope;
use crate::project_slice::gcode_emit::motion::extrusion;
use crate::project_slice::gcode_emit::motion::{
    EmitState, arc,
    features::PathProperties,
    format::{axis as format_axis, extrusion as format_extrusion, z as format_z},
};

pub(in crate::project_slice::gcode_emit::motion) fn segments(
    output: &mut Vec<u8>,
    points: &[(f64, f64)],
    slope: Slope,
    properties: PathProperties<'_>,
    state: &mut EmitState,
) {
    let total_length = points
        .windows(2)
        .map(|pair| (pair[1].0 - pair[0].0).hypot(pair[1].1 - pair[0].1))
        .sum::<f64>();
    let mut path_length = 0.0;
    for pair in points.windows(2) {
        let length = (pair[1].0 - pair[0].0).hypot(pair[1].1 - pair[0].1);
        if length < super::super::path::SOURCE_EPSILON_MM {
            continue;
        }
        path_length += length;
        let ratio = path_length / total_length;
        let z_ratio = slope.z_begin + (slope.z_end - slope.z_begin) * ratio;
        let e_ratio = slope.e_begin + (slope.e_end - slope.e_begin) * ratio;
        let z =
            state.layer_z - f64::from(properties.height) + f64::from(properties.height) * z_ratio;
        let extrusion = extrusion::for_length(
            length,
            properties.mm3_per_mm,
            state.options.filament_flow_ratio,
            state.options.print_flow_ratio,
            state.options.filament_area,
        ) * slope.flow_ratio
            * state
                .small_area_flow
                .multiplier_for_feature(properties.feature, length)
            * e_ratio;
        let extrusion = extrusion::coordinate(state, extrusion);
        let end = arc::Point {
            x: pair[1].0,
            y: pair[1].1,
        };
        output.extend_from_slice(
            format!(
                "G1 X{} Y{} Z{} E{}\n",
                format_axis(end.x),
                format_axis(end.y),
                format_z(z),
                format_extrusion(extrusion)
            )
            .as_bytes(),
        );
        state.x = end.x;
        state.y = end.y;
        state.scarf_z = Some(z);
        state.wipe_start = Some(end);
    }
}
