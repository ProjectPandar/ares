mod arc;
mod clip;
mod features;
mod format;
mod loop_paths;
mod options;
mod overhang;
mod path;
#[cfg(test)]
mod tests;
mod travel;

pub(in crate::project_slice) use arc::simplify_points;

use features::PathProperties;
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
    pub(super) previous_layer_boundary: Option<&'a crate::geometry::LineDistanceTree<'a>>,
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
                        path::emit(
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
                                fitting: &[],
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
    path::emit(
        output,
        path.polyline.points.iter().map(|point| (point.x, point.y)),
        PathProperties {
            mm3_per_mm: path.mm3_per_mm,
            width: path.width,
            height: path.height,
            feature,
            is_perimeter: path.role != ExtrusionRole::GapFill,
            end_clip,
            fitting: &path.polyline.fitting,
        },
        geometry,
        state,
    );
}
