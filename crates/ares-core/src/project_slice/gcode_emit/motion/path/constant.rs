use super::SOURCE_EPSILON_MM;
use crate::project_slice::gcode_emit::motion::{
    EmitState, arc, extrusion, fan,
    features::PathProperties,
    format::{push_axis, push_extrusion, push_offset},
};
use crate::project_slice::perimeters::classic::materialize::FittedMove;

#[inline]
fn scaled_segment_length(
    a: (i64, i64),
    b: (i64, i64),
    scale: crate::geometry::CoordinateScale,
) -> f64 {
    // Line::length() * SCALING_FACTOR: subtract before converting to mm.
    let dx = (b.0 - a.0) as f64;
    let dy = (b.1 - a.1) as f64;
    (dx * dx + dy * dy).sqrt() * scale.factor()
}

pub(super) struct Emission<'a, 'b> {
    pub(super) output: &'a mut Vec<u8>,
    pub(super) points: &'a [(f64, f64)],
    pub(super) local_points: &'a [(f64, f64)],
    pub(super) timelapse_context: crate::project_slice::gcode_emit::timelapse::Context<'b>,
    pub(super) uncut_lengths: &'a [f64],
    pub(super) scaled: (&'a [(i64, i64)], crate::geometry::CoordinateScale),
    pub(super) fitting: &'a [FittedMove],
    pub(super) last_scaled: (i64, i64),
    pub(super) path_overhang_fan_active: bool,
    pub(super) force_no_extrusion: bool,
    pub(super) properties: PathProperties<'b>,
    pub(super) state: &'a mut EmitState,
}

pub(super) fn emit(emission: Emission<'_, '_>) -> Result<(), crate::SliceError> {
    let Emission {
        output,
        points,
        local_points,
        timelapse_context,
        uncut_lengths,
        scaled: (scaled_points, scale),
        fitting,
        last_scaled,
        path_overhang_fan_active,
        force_no_extrusion,
        properties,
        state,
    } = emission;

    fan::update_for_constant_path(output, properties, path_overhang_fan_active, state);
    if !state.options.enable_arc_fitting || fitting.is_empty() {
        for endpoint in 1..points.len() {
            let length =
                scaled_segment_length(scaled_points[endpoint - 1], scaled_points[endpoint], scale);
            if length < SOURCE_EPSILON_MM {
                continue;
            }
            let end = arc::Point {
                x: points[endpoint].0,
                y: points[endpoint].1,
            };
            if force_no_extrusion {
                extrusion::force_no_extrusion_segment(output, end, state);
            } else {
                state.mark_belt_line(
                    output,
                    properties.width,
                    (scaled_points[endpoint - 1], scaled_points[endpoint]),
                    scale,
                );
                let uncut_length = uncut_lengths.get(endpoint - 1).copied().unwrap_or(length);
                extrusion::linear_segment(output, end, (length, uncut_length), properties, state);
            }
            let (x, y) = local_points[endpoint];
            crate::project_slice::gcode_emit::layers::farthest::check_and_insert(
                output,
                state,
                (x + state.origin.0, y + state.origin.1),
                timelapse_context,
            )?;
        }
        output.extend_from_slice(b";_EXTRUDE_END\n");
        state.last_scaled_position = last_scaled;
        state.last_position_defined = true;
        state.scarf_z = Some(state.layer_z);
        return Ok(());
    }

    let arc_points = points
        .iter()
        .map(|&(x, y)| arc::Point { x, y })
        .collect::<Vec<_>>();
    // Upstream replays the precomputed fitting result at export time
    // (`GCode.cpp:6992`); paths with no fitting (brim, skirt, clipped
    // tails) emit straight segments and are never fitted here.
    let segments = arc::from_fitting(&arc_points, fitting, state.origin, state.extruder_offset);
    let endpoints = fitting.iter().flat_map(|fitted| {
        let first = if fitted.arc.is_some() {
            fitted.end
        } else {
            fitted.start + 1
        };
        first..=fitted.end
    });
    for (segment, endpoint) in segments.into_iter().zip(endpoints) {
        match segment {
            arc::Segment::Line { end, length } if length >= SOURCE_EPSILON_MM => {
                if force_no_extrusion {
                    extrusion::force_no_extrusion_segment(output, end, state);
                } else {
                    state.mark_belt_line(
                        output,
                        properties.width,
                        (scaled_points[endpoint - 1], scaled_points[endpoint]),
                        scale,
                    );
                    let uncut_length = uncut_lengths.get(endpoint - 1).copied().unwrap_or(length);
                    extrusion::linear_segment(
                        output,
                        end,
                        (length, uncut_length),
                        properties,
                        state,
                    );
                }
                let (x, y) = local_points[endpoint];
                crate::project_slice::gcode_emit::layers::farthest::check_and_insert(
                    output,
                    state,
                    (x + state.origin.0, y + state.origin.1),
                    timelapse_context,
                )?;
            }
            arc::Segment::Line { .. } => {}
            arc::Segment::Arc(arc_segment) if arc_segment.length >= SOURCE_EPSILON_MM => {
                let extrusion = extrusion::for_length(
                    arc_segment.length,
                    extrusion::effective_flow_at_emission(properties, state),
                    state.options.filament_flow_ratio,
                    state.options.filament_area,
                ) * extrusion::small_area_multiplier(
                    state,
                    properties.feature,
                    arc_segment.length,
                );
                let extrusion = extrusion::coordinate(state, extrusion);
                output.extend_from_slice(if arc_segment.clockwise {
                    b"G2 X"
                } else {
                    b"G3 X"
                });
                push_axis(output, arc_segment.end.x - state.xy_offset.0);
                output.extend_from_slice(b" Y");
                push_axis(output, arc_segment.end.y - state.xy_offset.1);
                output.extend_from_slice(b" I");
                push_offset(output, arc_segment.center.x - arc_segment.start.x);
                output.extend_from_slice(b" J");
                push_offset(output, arc_segment.center.y - arc_segment.start.y);
                output.extend_from_slice(b" E");
                push_extrusion(output, extrusion);
                extrusion::push_feature_comment(
                    output,
                    state.options.gcode_comments,
                    properties.feature,
                );
                output.push(b'\n');
                state.x = arc_segment.end.x;
                state.y = arc_segment.end.y;
                state.wipe_start = Some(arc_segment.end);
                // GCode.cpp:10174: the farthest-point check at the fitted arc end.
                let (x, y) = local_points[endpoint];
                crate::project_slice::gcode_emit::layers::farthest::check_and_insert(
                    output,
                    state,
                    (x + state.origin.0, y + state.origin.1),
                    timelapse_context,
                )?;
            }
            arc::Segment::Arc(_) => {}
        }
    }
    output.extend_from_slice(b";_EXTRUDE_END\n");

    state.last_scaled_position = last_scaled;
    state.last_position_defined = true;
    state.scarf_z = Some(state.layer_z);
    Ok(())
}
