mod arc;
mod clip;
mod features;
mod format;
mod loop_paths;
mod options;
#[cfg(test)]
mod tests;
mod travel;

use features::PathProperties;
use format::{axis as format_axis, extrusion as format_extrusion};
pub(in crate::project_slice::gcode_emit) use options::MotionOptions;
#[cfg(test)]
pub(in crate::project_slice::gcode_emit) use options::first_nullable_float;

use super::super::island_print_order::{IslandPrintEntity, OrderedExtrusionLayer};
use crate::SliceError;

#[derive(Default)]
pub(super) struct EmitState {
    pub(super) x: f64,
    pub(super) y: f64,
    pub(super) offset: (f64, f64),
    pub(super) travel_feedrate: f64,
    pub(super) extrusion_feedrate: f64,
    pub(super) options: MotionOptions,
    pub(super) layer_index: usize,
    pub(super) positioned: bool,
    pub(super) last_feature: Option<&'static str>,
    pub(super) last_width: Option<f32>,
    pub(super) last_height: Option<f32>,
    pub(super) last_acceleration: Option<u32>,
    pub(super) layer_z: f64,
    pub(super) retracted: bool,
    pub(super) wipe_path: Vec<arc::Point>,
    pub(super) lifted: bool,
}
#[derive(Clone, Copy)]
pub(super) struct LayerGeometry<'a> {
    pub(super) internal_surfaces: &'a [crate::project_slice::region_slices::RegionSurface],
    pub(super) scale: crate::geometry::CoordinateScale,
}

pub(super) fn begin_layer(
    output: &mut Vec<u8>,
    state: &mut EmitState,
    layer_index: usize,
    layer_z: f64,
    layer_height: f64,
) {
    state.layer_index = layer_index;
    state.last_height = Some(layer_height as f32);
    state.layer_z = layer_z;
    state.travel_feedrate = if layer_index == 0 {
        state.options.first_layer_travel_feedrate
    } else {
        state.options.travel_feedrate
    };
    let acceleration = if layer_index == 0 {
        state.options.initial_layer_acceleration
    } else {
        state.options.default_acceleration
    };
    set_acceleration(output, state, acceleration);
}

pub(super) fn begin_object_travel(output: &mut Vec<u8>, state: &mut EmitState) {
    let acceleration = if state.layer_index == 0 {
        state.options.initial_layer_travel_acceleration
    } else {
        state.options.travel_acceleration
    };
    set_acceleration(output, state, acceleration);
}

pub(super) fn end_layer_for_timelapse(output: &mut Vec<u8>, state: &mut EmitState) {
    if state.options.retract_when_changing_layer && state.positioned {
        travel::retract_for_timelapse(output, state);
    }
}

fn set_acceleration(output: &mut Vec<u8>, state: &mut EmitState, acceleration: u32) {
    if state.last_acceleration != Some(acceleration) {
        output.extend_from_slice(format!("M204 S{acceleration}\n").as_bytes());
        state.last_acceleration = Some(acceleration);
    }
}

#[expect(
    clippy::excessive_nesting,
    reason = "keeps the source ordered extrusion-entity traversal together"
)]
pub(super) fn emit_layer(
    output: &mut Vec<u8>,
    layer: &OrderedExtrusionLayer,
    geometry: LayerGeometry<'_>,
    state: &mut EmitState,
) -> Result<(), SliceError> {
    for island in &layer.islands {
        for entity in &island.entities {
            match entity {
                IslandPrintEntity::Perimeter(collection) => {
                    for loop_ in &collection.entities {
                        loop_paths::emit(
                            output,
                            &loop_.extrusion_loop.paths,
                            geometry,
                            state,
                        );
                    }
                }
                IslandPrintEntity::Fill(collection) => {
                    for path in &collection.paths {
                        emit_points(
                            output,
                            path.polyline.points().iter().map(|point| (point.x(), point.y())),
                            PathProperties {
                                mm3_per_mm: path.mm3_per_mm,
                                width: path.width,
                                height: path.height,
                                feature: features::for_fill(path.role),
                                is_perimeter: matches!(
                                    path.role,
                                    crate::ExtrusionRole::Perimeter
                                        | crate::ExtrusionRole::ExternalPerimeter
                                        | crate::ExtrusionRole::OverhangPerimeter
                                ),
                                end_clip: 0.0,
                            },
                            geometry,
                            state,
                        );
                    }
                }
                IslandPrintEntity::Thin(entity) => match entity {
                    crate::project_slice::perimeters::classic::gap_extrusion::GapFillEntity::Path(path) => {
                        emit_materialized_path(output, path, 0.0, geometry, state);
                    }
                    crate::project_slice::perimeters::classic::gap_extrusion::GapFillEntity::Loop(paths) => {
                        loop_paths::emit(output, paths, geometry, state);
                    }
                },
            }
        }
    }
    Ok(())
}

fn emit_materialized_path(
    output: &mut Vec<u8>,
    path: &crate::project_slice::perimeters::classic::materialize::ExtrusionPath,
    end_clip: f64,
    geometry: LayerGeometry<'_>,
    state: &mut EmitState,
) {
    use crate::project_slice::perimeters::classic::materialize::ExtrusionRole;
    let feature = match path.role {
        ExtrusionRole::ExternalPerimeter => "Outer wall",
        ExtrusionRole::Perimeter => "Inner wall",
        ExtrusionRole::OverhangPerimeter => "Overhang wall",
        ExtrusionRole::GapFill => "Gap infill",
    };
    emit_points(
        output,
        path.polyline.points.iter().map(|point| (point.x, point.y)),
        PathProperties {
            mm3_per_mm: path.mm3_per_mm,
            width: path.width,
            height: path.height,
            feature,
            is_perimeter: path.role != ExtrusionRole::GapFill,
            end_clip,
        },
        geometry,
        state,
    );
}

fn emit_points(
    output: &mut Vec<u8>,
    points: impl Iterator<Item = (i64, i64)>,
    properties: PathProperties,
    geometry: LayerGeometry<'_>,
    state: &mut EmitState,
) {
    let mut points = points
        .map(|(x, y)| {
            (
                geometry.scale.unscale(x) + state.offset.0,
                geometry.scale.unscale(y) + state.offset.1,
            )
        })
        .collect::<Vec<_>>();
    clip::clip_end(&mut points, properties.end_clip);
    let Some(&(first_x, first_y)) = points.first() else {
        return;
    };
    let first_position = !state.positioned;
    let needs_travel = first_position || first_x != state.x || first_y != state.y;
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
    let (acceleration, speed) = properties.kinematics(&state.options, state.layer_index);
    set_acceleration(output, state, acceleration);
    state.extrusion_feedrate = speed.min(
        state.options.max_volumetric_speed
            / (properties.mm3_per_mm * state.options.filament_flow_ratio),
    ) * 60.0;
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
                super::format_processor_float(f64::from(properties.height))
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
