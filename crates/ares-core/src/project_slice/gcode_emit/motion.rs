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
    pub(super) filament_area: f64,
    pub(super) travel_feedrate: f64,
    pub(super) extrusion_feedrate: f64,
    pub(super) positioned: bool,
    pub(super) last_feature: Option<&'static str>,
    pub(super) last_width: Option<f32>,
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
) -> Result<(), SliceError> {
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

struct PathProperties {
    mm3_per_mm: f64,
    width: f32,
    feature: &'static str,
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
    let mut points = points.map(|(x, y)| {
        (
            scale.unscale(x) + state.offset.0,
            scale.unscale(y) + state.offset.1,
        )
    });
    let Some((first_x, first_y)) = points.next() else {
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
    for (x, y) in points {
        let distance = (x - state.x).hypot(y - state.y);
        let extrusion = distance * properties.mm3_per_mm / state.filament_area;
        output.extend_from_slice(
            format!(
                "G1 X{} Y{} E{}\n",
                format_axis(x),
                format_axis(y),
                format_extrusion(extrusion)
            )
            .as_bytes(),
        );
        state.x = x;
        state.y = y;
    }
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
