use crate::project_slice::gcode_emit::motion::{
    EmitState, arc, extrusion, features::PathProperties,
};

/// G-code-frame and local path points, index-aligned.
type PathPoints<'a> = (&'a [(f64, f64)], &'a [(f64, f64)]);

pub(in crate::project_slice::gcode_emit::motion) fn segments(
    output: &mut Vec<u8>,
    (points, local_points): PathPoints<'_>,
    properties: PathProperties<'_>,
    state: &mut EmitState,
    timelapse_context: crate::project_slice::gcode_emit::timelapse::Context<'_>,
) -> Result<(), crate::SliceError> {
    let slope = properties.slope.unwrap();
    let total_length = points
        .windows(2)
        .map(|pair| (pair[1].0 - pair[0].0).hypot(pair[1].1 - pair[0].1))
        .sum::<f64>();
    let mut path_length = 0.0;
    for (index, pair) in points.windows(2).enumerate() {
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
        let delta = extrusion::for_length(
            length,
            extrusion::effective_flow(properties, &state.options, state.on_first_layer),
            state.options.filament_flow_ratio,
            state.options.filament_area,
        ) * extrusion::small_area_multiplier(state, properties.feature, length)
            * e_ratio;
        extrusion::extrude_to_xyz(
            output,
            extrusion::XyzEmission {
                end: arc::Point {
                    x: pair[1].0,
                    y: pair[1].1,
                },
                z,
                delta,
                properties,
            },
            state,
        );
        // GCode.cpp:10118: the farthest-point check after each sloped line.
        let (x, y) = local_points[index + 1];
        crate::project_slice::gcode_emit::layers::farthest::check_and_insert(
            output,
            state,
            (x + state.origin.0, y + state.origin.1),
            timelapse_context,
        )?;
    }
    Ok(())
}
