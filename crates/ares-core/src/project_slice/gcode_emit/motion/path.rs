use super::{
    EmitState, LayerGeometry, arc, begin_object_travel, clip,
    features::PathProperties,
    format::{axis as format_axis, extrusion as format_extrusion},
    overhang, set_acceleration, travel,
};

pub(super) fn emit(
    output: &mut Vec<u8>,
    points: impl Iterator<Item = (i64, i64)>,
    properties: PathProperties,
    geometry: LayerGeometry<'_>,
    state: &mut EmitState,
) {
    let mut local_points = points
        .map(|(x, y)| (geometry.scale.unscale(x), geometry.scale.unscale(y)))
        .collect::<Vec<_>>();
    clip::clip_end(&mut local_points, properties.end_clip);
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
    let Some(&(first_x, first_y)) = points.first() else {
        return;
    };
    let first_position = !state.positioned;
    let needs_travel = first_position
        || quantize_axis(first_x) != quantize_axis(state.x)
        || quantize_axis(first_y) != quantize_axis(state.y);
    let feedrate_interrupted = needs_travel || state.retracted;
    let previous_extrusion_feedrate = state.extrusion_feedrate;
    if needs_travel {
        begin_object_travel(output, state);
        let inside_internal_surface = state.options.reduce_infill_retraction
            && !properties.is_perimeter
            && travel::inside_internal_surfaces(
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
        let retract = !first_position
            && !state.retracted
            && (first_x - state.x).hypot(first_y - state.y)
                >= state.options.retraction_minimum_travel
            && !inside_internal_surface;
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
        output.extend_from_slice(
            format!(
                "G1 X{} Y{} F{}\n",
                format_axis(first_x),
                format_axis(first_y),
                format_axis(state.travel_feedrate)
            )
            .as_bytes(),
        );
        state.x = first_x;
        state.y = first_y;
        state.positioned = true;
    }
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
        output.extend_from_slice(format!("; FEATURE: {}\n", properties.feature).as_bytes());
        state.last_feature = Some(properties.feature);
    }
    if state.last_width != Some(properties.width) {
        output.extend_from_slice(
            format!(
                "; LINE_WIDTH: {}\n",
                format_axis(f64::from(properties.width))
            )
            .as_bytes(),
        );
        state.last_width = Some(properties.width);
    }
    if state
        .last_height
        .is_none_or(|height| (height - properties.height).abs() > f32::EPSILON)
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
    if feedrate_interrupted
        || (state.extrusion_feedrate - previous_extrusion_feedrate).abs() > f64::EPSILON
    {
        output.extend_from_slice(
            format!("G1 F{}\n", format_axis(state.extrusion_feedrate)).as_bytes(),
        );
    }
    if let Some(processed) = processed {
        emit_variable_segments(VariableEmission {
            output,
            points: &points,
            processed: &processed,
            original_speed,
            properties,
            state,
        });
        return;
    }
    let arc_points = points
        .iter()
        .map(|&(x, y)| arc::Point { x, y })
        .collect::<Vec<_>>();
    let segments = if state.options.enable_arc_fitting {
        arc::fit(&arc_points, state.options.arc_fitting_tolerance)
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
    let mut emitted_path = Vec::with_capacity(segments.len() + 1);
    emitted_path.push(arc::Point {
        x: first_x,
        y: first_y,
    });
    for segment in segments {
        let end = match segment {
            arc::Segment::Line { end, length } => {
                emit_linear_segment(output, end, length, properties, state);
                end
            }
            arc::Segment::Arc(arc_segment) => {
                let extrusion =
                    arc_segment.length * properties.mm3_per_mm * state.options.filament_flow_ratio
                        / state.options.filament_area;
                let command = if arc_segment.clockwise { "G2" } else { "G3" };
                output.extend_from_slice(
                    format!(
                        "{command} X{} Y{} I{} J{} E{}\n",
                        format_axis(arc_segment.end.x),
                        format_axis(arc_segment.end.y),
                        format_axis(arc_segment.center.x - state.x),
                        format_axis(arc_segment.center.y - state.y),
                        format_extrusion(extrusion)
                    )
                    .as_bytes(),
                );
                state.x = arc_segment.end.x;
                state.y = arc_segment.end.y;
                arc_segment.end
            }
        };
        emitted_path.push(end);
    }
    state.wipe_path = emitted_path;
}

fn emit_linear_segment(
    output: &mut Vec<u8>,
    end: arc::Point,
    length: f64,
    properties: PathProperties,
    state: &mut EmitState,
) {
    let extrusion = length * properties.mm3_per_mm * state.options.filament_flow_ratio
        / state.options.filament_area;
    output.extend_from_slice(
        format!(
            "G1 X{} Y{} E{}\n",
            format_axis(end.x),
            format_axis(end.y),
            format_extrusion(extrusion)
        )
        .as_bytes(),
    );
    state.x = end.x;
    state.y = end.y;
}
struct VariableEmission<'a> {
    output: &'a mut Vec<u8>,
    points: &'a [(f64, f64)],
    processed: &'a [overhang::ProcessedPoint],
    original_speed: f64,
    properties: PathProperties,
    state: &'a mut EmitState,
}

fn emit_variable_segments(command: VariableEmission<'_>) {
    let VariableEmission {
        output,
        points,
        processed,
        original_speed,
        properties,
        state,
    } = command;
    let original_feedrate = original_speed * 60.0;
    let mut last_feedrate = processed[0].speed * 60.0;
    let mut emitted_path = Vec::with_capacity(points.len());
    emitted_path.push(arc::Point {
        x: points[0].0,
        y: points[0].1,
    });
    for index in 1..points.len() {
        let feedrate = processed[index - 1].speed * 60.0;
        if (last_feedrate - feedrate).abs() > 60.0 {
            output.extend_from_slice(format!("G1 F{}\n", format_axis(feedrate)).as_bytes());
            last_feedrate = feedrate;
        } else if (original_feedrate - feedrate).abs() <= 60.0 {
            output
                .extend_from_slice(format!("G1 F{}\n", format_axis(original_feedrate)).as_bytes());
            last_feedrate = original_feedrate;
        }
        let end = arc::Point {
            x: points[index].0,
            y: points[index].1,
        };
        let length = (end.x - state.x).hypot(end.y - state.y);
        emit_linear_segment(output, end, length, properties, state);
        emitted_path.push(end);
    }
    state.extrusion_feedrate = last_feedrate;
    state.wipe_path = emitted_path;
}

fn quantize_axis(value: f64) -> f64 {
    (value * 1_000.0).round() / 1_000.0
}
