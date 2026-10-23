use super::SOURCE_EPSILON_MM;
use crate::project_slice::gcode_emit::motion::{
    EmitState, arc, extrusion, fan,
    features::{PathProperties, feature_description, is_bridge_feature},
    format::{axis as format_axis, extrusion as format_extrusion, offset as format_offset},
};
use crate::project_slice::perimeters::classic::materialize::FittedMove;

pub(super) struct Emission<'a, 'b> {
    pub(super) output: &'a mut Vec<u8>,
    pub(super) points: &'a [(f64, f64)],
    // The RAW pre-arc-fit path points (upstream `path.polyline`,
    // `GCode.cpp:6111`) used to build the wipe path — not the post
    // `clip_fitting_end` `local_points`.
    pub(super) wipe_points: &'a [(f64, f64)],
    pub(super) segment_lengths: &'a [f64],
    pub(super) fitting: &'a [FittedMove],
    pub(super) last_scaled: (i64, i64),
    pub(super) force_no_extrusion: bool,
    pub(super) properties: PathProperties<'b>,
    pub(super) state: &'a mut EmitState,
}

pub(super) fn emit(emission: Emission<'_, '_>) {
    let Emission {
        output,
        points,
        wipe_points,
        segment_lengths,
        fitting,
        last_scaled,
        force_no_extrusion,
        properties,
        state,
    } = emission;
    let wipe_points = wipe_points
        .iter()
        .map(|&(x, y)| arc::Point { x, y })
        .collect::<Vec<_>>();
    fan::update_for_constant_path(output, properties, state);
    let arc_points = points
        .iter()
        .map(|&(x, y)| arc::Point { x, y })
        .collect::<Vec<_>>();
    let segments = if state.options.enable_arc_fitting && !fitting.is_empty() {
        // Upstream replays the precomputed fitting result at export time
        // (`GCode.cpp:6992`); paths with no fitting (brim, skirt, clipped
        // tails) emit straight segments and are never fitted here.
        arc::from_fitting(&arc_points, fitting, state.offset)
    } else {
        points
            .windows(2)
            .zip(segment_lengths.iter())
            .map(|(pair, &length)| arc::Segment::Line {
                end: arc::Point {
                    x: pair[1].0,
                    y: pair[1].1,
                },
                length,
            })
            .collect()
    };
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
    for segment in segments {
        match segment {
            arc::Segment::Line { end, length } if length >= SOURCE_EPSILON_MM => {
                if force_no_extrusion {
                    extrusion::force_no_extrusion_segment(output, end, state);
                } else {
                    extrusion::linear_segment(output, end, length, properties, state);
                }
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
                let command = if arc_segment.clockwise { "G2" } else { "G3" };
                output.extend_from_slice(
                    format!(
                        "{command} X{} Y{} I{} J{} E{}{comment}\n",
                        format_axis(arc_segment.end.x),
                        format_axis(arc_segment.end.y),
                        format_offset(arc_segment.center.x - arc_segment.start.x),
                        format_offset(arc_segment.center.y - arc_segment.start.y),
                        format_extrusion(extrusion)
                    )
                    .as_bytes(),
                );
                state.x = arc_segment.end.x;
                state.y = arc_segment.end.y;
                state.wipe_start = Some(arc_segment.end);
            }
            arc::Segment::Arc(_) => {}
        }
    }
    output.extend_from_slice(b";_EXTRUDE_END\n");
    state.wipe_path = wipe_points.into_iter().rev().collect();
    state.last_scaled_position = last_scaled;
    state.last_position_defined = true;
    state.scarf_z = Some(state.layer_z);
}
