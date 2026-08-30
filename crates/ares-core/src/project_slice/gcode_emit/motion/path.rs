mod retraction;
mod start_travel;
mod travel_emit;
mod variable;

pub(super) use retraction::can_skip_retraction;

use super::{
    EmitState, LayerGeometry, arc, clip, extrusion, fan,
    features::PathProperties,
    format::{axis as format_axis, extrusion as format_extrusion, offset as format_offset},
    overhang, set_accel_and_jerk,
};

const SOURCE_EPSILON_MM: f64 = 1e-4;

pub(super) fn emit(
    output: &mut Vec<u8>,
    points: impl Iterator<Item = (i64, i64)>,
    properties: PathProperties<'_>,
    geometry: LayerGeometry<'_>,
    state: &mut EmitState,
) {
    let mut scaled_points = points.collect::<Vec<_>>();
    let source_length = scaled_points
        .windows(2)
        .map(|pair| {
            let dx = (pair[1].0 - pair[0].0) as f64;
            let dy = (pair[1].1 - pair[0].1) as f64;
            dx.hypot(dy) * geometry.scale.factor()
        })
        .sum();
    clip::clip_end(
        &mut scaled_points,
        properties.end_clip / geometry.scale.factor(),
    );
    let Some((&first_scaled, &last_scaled)) = scaled_points.first().zip(scaled_points.last())
    else {
        return;
    };
    let mut local_points = scaled_points
        .into_iter()
        .map(|(x, y)| (geometry.scale.unscale(x), geometry.scale.unscale(y)))
        .collect::<Vec<_>>();
    let mut fitting = properties.fitting.to_vec();
    if properties.end_clip > 0.0 {
        arc::clip_fitting_end(&mut local_points, &mut fitting, geometry.scale);
    }
    let (acceleration, configured_speed) =
        properties.kinematics(&state.options, state.layer_index, source_length);
    let original_speed = configured_speed.min(
        state.options.max_volumetric_speed
            / (properties.mm3_per_mm * state.options.filament_flow_ratio),
    );
    let processed = overhang::estimate(overhang::EstimateRequest {
        points: &local_points,
        properties,
        geometry,
        options: &state.options,
        layer_index: state.layer_index,
        original_speed,
    });
    let points = processed.as_ref().map_or_else(
        || {
            local_points
                .iter()
                .map(|&(x, y)| (x + state.offset.0, y + state.offset.1))
                .collect::<Vec<_>>()
        },
        |points| {
            points
                .iter()
                .map(|point| {
                    (
                        travel_emit::quantize_axis(point.x + state.offset.0),
                        travel_emit::quantize_axis(point.y + state.offset.1),
                    )
                })
                .collect::<Vec<_>>()
        },
    );
    // Source travels to the raw path start before overhang processing quantizes its points.
    let Some(&(first_local_x, first_local_y)) = local_points.first() else {
        return;
    };
    let first_x = first_local_x + state.offset.0;
    let first_y = first_local_y + state.offset.1;
    start_travel::emit(
        output,
        state,
        start_travel::Request {
            first_scaled,
            first_x,
            first_y,
            properties,
            geometry,
        },
    );
    let jerk = properties.jerk(&state.options, state.layer_index);
    set_accel_and_jerk(output, state, acceleration, jerk, false);
    state.extrusion_feedrate = processed
        .as_ref()
        .map_or(original_speed, |points| points[0].speed)
        * 60.0;
    if state.last_feature != Some(properties.feature) {
        let feature = state.tags.feature(properties.feature) + "\n";
        output.extend_from_slice(feature.as_bytes());
        state.last_feature = Some(properties.feature);
    }
    if state.last_width != Some(properties.width) {
        output.extend_from_slice(
            format!(
                "{}\n",
                state
                    .tags
                    .width(&super::super::format_processor_float(f64::from(
                        properties.width
                    )))
            )
            .as_bytes(),
        );
        state.last_width = Some(properties.width);
    }
    if state
        .last_height
        .is_none_or(|height| (height - properties.height).abs() > 0.000_1)
    {
        output.extend_from_slice(
            format!(
                "{}\n",
                state
                    .tags
                    .height(&super::super::format_processor_float(f64::from(
                        properties.height
                    )))
            )
            .as_bytes(),
        );
        state.last_height = Some(properties.height);
    }
    extrusion::speed(output, state.extrusion_feedrate, properties);
    state.current_feedrate = state.extrusion_feedrate;
    if let Some(target) = state.pending_wipe_before_external_target.take() {
        output.extend_from_slice(
            format!("G1 X{} Y{}\n", format_axis(target.x), format_axis(target.y)).as_bytes(),
        );
        state.x = target.x;
        state.y = target.y;
        state.wipe_start = Some(target);
    }
    if let Some(processed) = processed {
        variable::emit(variable::Emission {
            output,
            points: &points,
            wipe_points: &local_points,
            processed: &processed,
            original_speed,
            properties,
            state,
        });
        state.last_scaled_position = Some(last_scaled);
        output.extend_from_slice(b";_EXTRUDE_END\n");
        return;
    }
    let wipe_points = local_points
        .iter()
        .map(|&(x, y)| arc::Point {
            x: x + state.offset.0,
            y: y + state.offset.1,
        })
        .collect::<Vec<_>>();
    fan::update_for_constant_path(output, properties, state);
    let arc_points = points
        .iter()
        .map(|&(x, y)| arc::Point { x, y })
        .collect::<Vec<_>>();
    let segments = if state.options.enable_arc_fitting {
        if fitting.is_empty() {
            arc::fit(&arc_points, state.options.arc_fitting_tolerance)
        } else {
            arc::from_fitting(&arc_points, &fitting, state.offset)
        }
    } else {
        points
            .windows(2)
            .map(|pair| arc::Segment::Line {
                end: arc::Point {
                    x: pair[1].0,
                    y: pair[1].1,
                },
                length: (pair[1].0 - pair[0].0).hypot(pair[1].1 - pair[0].1),
            })
            .collect()
    };
    for segment in segments {
        match segment {
            arc::Segment::Line { end, length } if length >= SOURCE_EPSILON_MM => {
                extrusion::linear_segment(output, end, length, properties, state);
            }
            arc::Segment::Line { .. } => {}
            arc::Segment::Arc(arc_segment) if arc_segment.length >= SOURCE_EPSILON_MM => {
                let extrusion = extrusion::for_length(
                    arc_segment.length,
                    properties.mm3_per_mm,
                    state.options.filament_flow_ratio,
                    state.options.print_flow_ratio,
                    state.options.filament_area,
                ) * state
                    .small_area_flow
                    .multiplier_for_feature(properties.feature, arc_segment.length);
                let extrusion = extrusion::coordinate(state, extrusion);
                let command = if arc_segment.clockwise { "G2" } else { "G3" };
                output.extend_from_slice(
                    format!(
                        "{command} X{} Y{} I{} J{} E{}\n",
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
    state.last_scaled_position = Some(last_scaled);
}
