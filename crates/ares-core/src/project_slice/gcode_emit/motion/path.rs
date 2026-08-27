use super::{
    EmitState, LayerGeometry, append_object_start, arc, begin_path_travel, clip, extrusion, fan,
    features::PathProperties,
    format::{axis as format_axis, extrusion as format_extrusion, offset as format_offset},
    overhang, set_acceleration, travel,
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
    let (acceleration, configured_speed) = properties.kinematics(&state.options, state.layer_index);
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
    let needs_travel = first_position || state.last_scaled_position != Some(first_scaled);
    let travel_distance = (first_x - state.x).hypot(first_y - state.y);
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
            state.last_feature,
            properties.is_perimeter,
            inside_internal_surface,
        );
        let retract = !first_position
            && !state.retracted
            && travel_distance >= state.options.retraction_minimum_travel
            && !skip_retraction;
        if retract {
            travel::retract_and_lift(
                output,
                arc::Point {
                    x: first_x,
                    y: first_y,
                },
                state,
            );
        }
        append_object_start(output, state);
        if retract && state.lifted {
            output.extend_from_slice(
                format!(
                    "G1 X{} Y{} Z{}\n",
                    format_axis(first_x),
                    format_axis(first_y),
                    format_extrusion(state.layer_z + state.options.z_hop)
                )
                .as_bytes(),
            );
        } else if state.retracted && first_position && state.options.z_hop > 0.0 {
            // Already retracted without a lift (layer-start retraction):
            // lift first, then travel at the lifted height — the feedrate
            // persists onto the XY move (`GCodeWriter::travel_to_xyz`).
            output.extend_from_slice(
                format!(
                    "G1 Z{} F{}\n",
                    format_extrusion(state.layer_z + state.options.z_hop),
                    format_axis(state.travel_feedrate)
                )
                .as_bytes(),
            );
            output.extend_from_slice(
                format!("G1 X{} Y{}\n", format_axis(first_x), format_axis(first_y)).as_bytes(),
            );
        } else {
            output.extend_from_slice(
                format!(
                    "G1 X{} Y{} F{}\n",
                    format_axis(first_x),
                    format_axis(first_y),
                    format_axis(state.travel_feedrate)
                )
                .as_bytes(),
            );
        }
        state.x = first_x;
        state.y = first_y;
        state.last_scaled_position = Some(first_scaled);
        state.positioned = true;
    }
    append_object_start(output, state);
    if state.retracted {
        if first_position && state.options.z_hop > 0.0 {
            output.extend_from_slice(
                format!(
                    "G1 Z{}\n",
                    format_extrusion(state.layer_z + state.options.z_hop)
                )
                .as_bytes(),
            );
        }
        output.extend_from_slice(format!("G1 Z{}\n", format_extrusion(state.layer_z)).as_bytes());
        output.extend_from_slice(
            format!(
                "G1 E{} F{}\n",
                format_extrusion(state.options.retraction_length),
                format_axis(state.options.deretraction_feedrate)
            )
            .as_bytes(),
        );
        state.retracted = false;
        state.lifted = false;
    }
    set_acceleration(output, state, acceleration);
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
    if let Some(processed) = processed {
        emit_variable_segments(VariableEmission {
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
                );
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

struct VariableEmission<'a> {
    output: &'a mut Vec<u8>,
    points: &'a [(f64, f64)],
    wipe_points: &'a [(f64, f64)],
    processed: &'a [overhang::ProcessedPoint],
    original_speed: f64,
    properties: PathProperties<'a>,
    state: &'a mut EmitState,
}

fn emit_variable_segments(command: VariableEmission<'_>) {
    let VariableEmission {
        output,
        points,
        wipe_points,
        processed,
        original_speed,
        properties,
        state,
    } = command;
    let original_feedrate = original_speed * 60.0;
    let mut last_feedrate = processed[0].speed * 60.0;
    let mut previous = points[0];
    for index in 1..points.len() {
        let end = arc::Point {
            x: points[index].0,
            y: points[index].1,
        };
        fan::update_for_variable_segment(
            output,
            properties,
            processed[index - 1],
            processed[index],
            state,
        );
        let length = (end.x - previous.0).hypot(end.y - previous.1);
        if length < SOURCE_EPSILON_MM {
            continue;
        }
        let feedrate = processed[index - 1].speed * 60.0;
        if (last_feedrate - feedrate).abs() > 60.0 {
            extrusion::speed(output, feedrate, properties);
            last_feedrate = feedrate;
        } else if (original_feedrate - feedrate).abs() <= 60.0 {
            extrusion::speed(output, original_feedrate, properties);
            last_feedrate = original_feedrate;
        }
        extrusion::linear_segment(output, end, length, properties, state);
        previous = (end.x, end.y);
    }
    state.extrusion_feedrate = last_feedrate;
    state.wipe_start = wipe_points.last().map(|&(x, y)| arc::Point {
        x: x + state.offset.0,
        y: y + state.offset.1,
    });
    state.wipe_path = wipe_points
        .iter()
        .rev()
        .map(|&(x, y)| arc::Point {
            x: x + state.offset.0,
            y: y + state.offset.1,
        })
        .collect();
}

fn quantize_axis(value: f64) -> f64 {
    (value * 1_000.0).round() / 1_000.0
}

pub(super) fn can_skip_retraction(
    reduce_infill_retraction: bool,
    previous_feature: Option<&str>,
    current_is_perimeter: bool,
    inside_internal_surface: bool,
) -> bool {
    reduce_infill_retraction
        && !matches!(previous_feature, Some("Outer wall" | "Overhang wall"))
        && !current_is_perimeter
        && inside_internal_surface
}
