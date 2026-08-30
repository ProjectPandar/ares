mod travel_emit;
mod variable;

use super::{
    EmitState, LayerGeometry, append_object_start, arc, begin_path_travel, clip, extrusion, fan,
    features::PathProperties,
    format::{
        axis as format_axis, extrusion as format_extrusion, offset as format_offset, z as format_z,
    },
    overhang, set_accel_and_jerk, travel,
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
                        quantize_axis(point.x + state.offset.0),
                        quantize_axis(point.y + state.offset.1),
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
    let first_position = !state.positioned;
    let layer_change_travel = state.layer_change_travel_pending && !first_position;
    let needs_travel = first_position || state.last_scaled_position != Some(first_scaled);
    let travel_distance = (first_x - state.x).hypot(first_y - state.y);
    let mut travel_set_layer_z = false;
    if needs_travel {
        begin_path_travel(output, state, properties.feature, travel_distance);
        let inside_internal_surface = travel::inside_internal_surfaces(
            geometry.internal_surfaces,
            arc::Point {
                x: state.x,
                y: state.y,
            },
            arc::Point {
                x: first_x,
                y: first_y,
            },
            geometry.scale,
            state.offset,
        );
        let skip_retraction = can_skip_retraction(
            state.options.reduce_infill_retraction,
            state.options.has_sparse_infill,
            state.last_feature,
            properties.is_perimeter,
            inside_internal_surface,
        );
        let retract = !first_position
            && !state.retracted
            && travel_distance >= state.options.retraction_minimum_travel
            && !skip_retraction;
        if retract {
            travel::retract_and_lift(output, state);
        }
        append_object_start(output, state);
        travel::emit_pending_lift(
            output,
            arc::Point {
                x: first_x,
                y: first_y,
            },
            state,
        );
        if state.template_lifted && state.lifted && !first_position {
            travel_emit::xy(output, first_x, first_y, state.travel_feedrate);
            state.template_lifted = false;
        } else if state.lifted && !first_position {
            if (state.current_feedrate - state.travel_feedrate).abs() > f64::EPSILON {
                travel_emit::xyz(
                    output,
                    first_x,
                    first_y,
                    state.layer_z + state.options.z_hop,
                    state.travel_feedrate,
                );
            } else {
                output.extend_from_slice(
                    format!(
                        "G1 X{} Y{} Z{}\n",
                        format_axis(first_x),
                        format_axis(first_y),
                        format_z(state.layer_z + state.options.z_hop)
                    )
                    .as_bytes(),
                );
            }
        } else if layer_change_travel && state.retracted {
            if state.options.z_hop > 0.0 && uses_sloped_lift(state.options.z_hop_type) {
                travel_emit::xy(output, first_x, first_y, state.travel_feedrate);
            } else if state.lifted {
                output.extend_from_slice(
                    format!(
                        "G1 X{} Y{} Z{}\n",
                        format_axis(first_x),
                        format_axis(first_y),
                        format_z(state.layer_z + state.options.z_hop)
                    )
                    .as_bytes(),
                );
            } else {
                travel_emit::xyz(
                    output,
                    first_x,
                    first_y,
                    state.layer_z,
                    state.travel_feedrate,
                );
                travel_set_layer_z = true;
            }
        } else if state.retracted
            && first_position
            && state.options.z_hop > 0.0
            && travel::lift_is_allowed(state)
        {
            if state.options.z_hop > 0.0 && uses_sloped_lift(state.options.z_hop_type) {
                travel_emit::xy(output, first_x, first_y, state.travel_feedrate);
            } else {
                output.extend_from_slice(
                    format!(
                        "G1 Z{} F{}\n",
                        format_z(state.layer_z + state.options.z_hop),
                        format_axis(state.travel_feedrate)
                    )
                    .as_bytes(),
                );
                output.extend_from_slice(
                    format!("G1 X{} Y{}\n", format_axis(first_x), format_axis(first_y)).as_bytes(),
                );
                state.lifted = true;
            }
        } else if layer_change_travel {
            if state.options.z_hop > 0.0 && uses_sloped_lift(state.options.z_hop_type) {
                travel_emit::xy(output, first_x, first_y, state.travel_feedrate);
                output.extend_from_slice(format!("G1 Z{}\n", format_z(state.layer_z)).as_bytes());
            } else {
                travel_emit::xyz(
                    output,
                    first_x,
                    first_y,
                    state.layer_z,
                    state.travel_feedrate,
                );
            }
            travel_set_layer_z = true;
        } else {
            travel_emit::xy(output, first_x, first_y, state.travel_feedrate);
        }
        state.x = first_x;
        state.y = first_y;
        state.last_scaled_position = Some(first_scaled);
        state.positioned = true;
        state.current_feedrate = state.travel_feedrate;
    } else if layer_change_travel {
        output.extend_from_slice(
            format!(
                "G1 Z{} F{}\n",
                format_z(state.layer_z),
                format_axis(state.travel_feedrate)
            )
            .as_bytes(),
        );
        travel_set_layer_z = true;
        state.current_feedrate = state.travel_feedrate;
    }
    state.layer_change_travel_pending = false;
    append_object_start(output, state);
    if state.retracted {
        if first_position
            && state.options.z_hop > 0.0
            && !state.lifted
            && travel::lift_is_allowed(state)
        {
            output.extend_from_slice(
                format!("G1 Z{}\n", format_z(state.layer_z + state.options.z_hop)).as_bytes(),
            );
            state.lifted = true;
        }
        if state.lifted && !travel_set_layer_z {
            output.extend_from_slice(format!("G1 Z{}\n", format_z(state.layer_z)).as_bytes());
        }
        let retraction_length = state.options.retraction_length;
        let unretract = extrusion::coordinate(state, retraction_length);
        output.extend_from_slice(
            format!(
                "G1 E{} F{}\n",
                format_extrusion(unretract),
                format_axis(state.options.deretraction_feedrate)
            )
            .as_bytes(),
        );
        state.current_feedrate = state.options.deretraction_feedrate;
        state.retracted = false;
        state.lifted = false;
        state.template_lifted = false;
    }
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
                "; LAYER_HEIGHT: {}\n",
                super::super::format_processor_float(f64::from(properties.height))
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

fn uses_sloped_lift(z_hop_type: crate::ZHopType) -> bool {
    z_hop_type != crate::ZHopType::Normal
}

fn quantize_axis(value: f64) -> f64 {
    (value * 1_000.0).round() / 1_000.0
}

pub(super) fn can_skip_retraction(
    reduce_infill_retraction: bool,
    has_sparse_infill: bool,
    previous_feature: Option<&str>,
    current_is_perimeter: bool,
    inside_internal_surface: bool,
) -> bool {
    reduce_infill_retraction
        && has_sparse_infill
        && !matches!(previous_feature, Some("Outer wall" | "Overhang wall"))
        && !current_is_perimeter
        && inside_internal_surface
}
