mod arc;
mod options;

pub(in crate::project_slice::gcode_emit) use options::MotionOptions;
#[cfg(test)]
pub(in crate::project_slice::gcode_emit) use options::first_nullable_float;

use super::super::island_print_order::{IslandPrintEntity, OrderedExtrusionLayer};
use crate::{ExtrusionRole, SliceError};

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
    pub(super) last_acceleration: Option<u32>,
}

#[expect(
    clippy::excessive_nesting,
    reason = "keeps the source ordered extrusion-entity traversal together"
)]
pub(super) fn emit_layer(
    output: &mut Vec<u8>,
    layer: &OrderedExtrusionLayer,
    scale: crate::geometry::CoordinateScale,
    state: &mut EmitState,
    layer_index: usize,
) -> Result<(), SliceError> {
    state.layer_index = layer_index;
    for island in &layer.islands {
        for entity in &island.entities {
            match entity {
                IslandPrintEntity::Perimeter(collection) => {
                    for loop_ in &collection.entities {
                        for path in &loop_.extrusion_loop.paths {
                            emit_materialized_path(output, path, scale, state);
                        }
                    }
                }
                IslandPrintEntity::Fill(collection) => {
                    for path in &collection.paths {
                        emit_polyline(
                            output,
                            &path.polyline,
                            PathProperties {
                                mm3_per_mm: path.mm3_per_mm,
                                width: path.width,
                                feature: feature_for_fill(path.role),
                            },
                            scale,
                            state,
                        );
                    }
                }
                IslandPrintEntity::Thin(entity) => match entity {
                    crate::project_slice::perimeters::classic::gap_extrusion::GapFillEntity::Path(path) => {
                        emit_materialized_path(output, path, scale, state);
                    }
                    crate::project_slice::perimeters::classic::gap_extrusion::GapFillEntity::Loop(paths) => {
                        for path in paths {
                            emit_materialized_path(output, path, scale, state);
                        }
                    }
                },
            }
        }
    }
    Ok(())
}

#[derive(Clone, Copy)]
struct PathProperties {
    mm3_per_mm: f64,
    width: f32,
    feature: &'static str,
}

impl PathProperties {
    fn kinematics(self, options: &MotionOptions, layer_index: usize) -> (u32, f64) {
        if layer_index == 0 {
            let speed = if self.feature == "Bottom surface" {
                options.initial_layer_infill_speed
            } else {
                options.initial_layer_speed
            };
            return (options.initial_layer_acceleration, speed);
        }
        match self.feature {
            "Outer wall" => (options.outer_wall_acceleration, options.outer_wall_speed),
            "Top surface" => (options.top_surface_acceleration, options.top_surface_speed),
            "Sparse infill" => (options.default_acceleration, options.sparse_infill_speed),
            "Internal solid infill" => (
                options.default_acceleration,
                options.internal_solid_infill_speed,
            ),
            "Gap infill" => (options.default_acceleration, options.gap_infill_speed),
            _ => (options.default_acceleration, options.inner_wall_speed),
        }
    }
}

fn emit_materialized_path(
    output: &mut Vec<u8>,
    path: &crate::project_slice::perimeters::classic::materialize::ExtrusionPath,
    scale: crate::geometry::CoordinateScale,
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
            feature,
        },
        scale,
        state,
    );
}

fn emit_polyline(
    output: &mut Vec<u8>,
    polyline: &crate::geometry::Polyline,
    properties: PathProperties,
    scale: crate::geometry::CoordinateScale,
    state: &mut EmitState,
) {
    emit_points(
        output,
        polyline.points().iter().map(|point| (point.x(), point.y())),
        properties,
        scale,
        state,
    );
}

fn emit_points(
    output: &mut Vec<u8>,
    points: impl Iterator<Item = (i64, i64)>,
    properties: PathProperties,
    scale: crate::geometry::CoordinateScale,
    state: &mut EmitState,
) {
    let points = points
        .map(|(x, y)| {
            (
                scale.unscale(x) + state.offset.0,
                scale.unscale(y) + state.offset.1,
            )
        })
        .collect::<Vec<_>>();
    let Some(&(first_x, first_y)) = points.first() else {
        return;
    };
    if !state.positioned || first_x != state.x || first_y != state.y {
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
    let (acceleration, speed) = properties.kinematics(&state.options, state.layer_index);
    if state.last_acceleration != Some(acceleration) {
        output.extend_from_slice(format!("M204 S{acceleration}\n").as_bytes());
        state.last_acceleration = Some(acceleration);
    }
    state.extrusion_feedrate =
        speed.min(state.options.max_volumetric_speed / properties.mm3_per_mm) * 60.0;
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
    output.extend_from_slice(format!("G1 F{}\n", format_axis(state.extrusion_feedrate)).as_bytes());
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
    for segment in segments {
        match segment {
            arc::Segment::Line { end, length } => {
                emit_linear_segment(output, end, length, properties, state);
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
            }
        }
    }
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

fn feature_for_fill(role: ExtrusionRole) -> &'static str {
    match role {
        ExtrusionRole::InternalInfill => "Sparse infill",
        ExtrusionRole::SolidInfill => "Internal solid infill",
        ExtrusionRole::TopSolidInfill => "Top surface",
        ExtrusionRole::BottomSurface => "Bottom surface",
        ExtrusionRole::Ironing => "Ironing",
        ExtrusionRole::BridgeInfill | ExtrusionRole::InternalBridgeInfill => "Bridge",
        ExtrusionRole::GapFill => "Gap infill",
        ExtrusionRole::Skirt => "Skirt",
        ExtrusionRole::Brim => "Brim",
        ExtrusionRole::SupportMaterial => "Support",
        ExtrusionRole::SupportMaterialInterface => "Support interface",
        ExtrusionRole::SupportTransition => "Support transition",
        ExtrusionRole::WipeTower => "Prime tower",
        ExtrusionRole::Custom => "Custom",
        ExtrusionRole::Perimeter => "Inner wall",
        ExtrusionRole::ExternalPerimeter => "Outer wall",
        ExtrusionRole::OverhangPerimeter => "Overhang wall",
        ExtrusionRole::None | ExtrusionRole::Mixed => "Mixed",
    }
}

fn format_axis(value: f64) -> String {
    let mut value = format!("{value:.3}");
    while value.ends_with('0') {
        value.pop();
    }
    if value.ends_with('.') {
        value.pop();
    }
    value
}

fn format_extrusion(value: f64) -> String {
    let value = format!("{value:.5}");
    value.strip_prefix('0').unwrap_or(&value).to_owned()
}
