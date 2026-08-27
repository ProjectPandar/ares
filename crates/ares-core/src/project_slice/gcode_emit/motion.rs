mod arc;
mod clip;
mod extrusion;
mod fan;
mod features;
mod format;
mod loop_paths;
mod options;
mod overhang;
mod path;
#[cfg(test)]
#[path = "motion/path/tests.rs"]
mod path_tests;
#[cfg(test)]
mod tests;
mod travel;

pub(in crate::project_slice) use arc::simplify_points;

use features::PathProperties;
pub(in crate::project_slice::gcode_emit) use options::MotionOptions;
#[cfg(test)]
pub(in crate::project_slice::gcode_emit) use options::first_nullable_float;

use super::super::island_print_order::{IslandPrintEntity, OrderedExtrusionLayer};
use crate::{
    SliceError,
    geometry::Point,
    project_slice::{
        fill_entities::FillExtrusionEntity,
        perimeters::classic::{
            gap_extrusion::GapFillEntity, shortest_path::chain_and_reorder_entities,
        },
    },
};

#[derive(Default)]
pub(super) struct EmitState {
    pub(super) x: f64,
    pub(super) y: f64,
    pub(super) offset: (f64, f64),
    pub(super) scale_factor: f64,
    pub(super) travel_feedrate: f64,
    pub(super) extrusion_feedrate: f64,
    pub(super) options: MotionOptions,
    pub(super) layer_index: usize,
    pub(super) positioned: bool,
    pub(super) last_scaled_position: Option<(i64, i64)>,
    pub(super) last_feature: Option<&'static str>,
    pub(super) last_width: Option<f32>,
    pub(super) last_height: Option<f32>,
    pub(super) last_acceleration: Option<u32>,
    pub(super) layer_z: f64,
    pub(super) retracted: bool,
    pub(super) wipe_path: Vec<arc::Point>,
    pub(super) wipe_start: Option<arc::Point>,
    pub(super) lifted: bool,
    pub(super) filament_used: f64,
    pub(super) part_fan_speed: u8,
    pub(super) physical_fan_speed: u8,
    pub(super) overhang_fan_active: bool,
    pub(super) overhang_fan_marker_layer: Option<usize>,
    pub(super) internal_bridge_fan_active: bool,
    pub(super) internal_bridge_fan_marker_layer: Option<usize>,
    pub(super) pending_object_start: Option<(u32, [u8; 12])>,
    pub(super) tags: super::tags::Tags,
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
    let acceleration = match layer_index {
        0 => Some(state.options.initial_layer_acceleration),
        1 => Some(state.options.default_acceleration),
        _ => None,
    };
    if let Some(acceleration) = acceleration {
        set_acceleration(output, state, acceleration);
    }
}

pub(super) fn queue_object_start(state: &mut EmitState, label_id: u32, encoded_labels: [u8; 12]) {
    state.pending_object_start = Some((label_id, encoded_labels));
}

fn append_object_start(output: &mut Vec<u8>, state: &mut EmitState) {
    let Some((label_id, encoded_labels)) = state.pending_object_start.take() else {
        return;
    };
    output.extend_from_slice(
        format!("; start printing object, unique label id: {label_id}\nM624 ").as_bytes(),
    );
    output.extend_from_slice(&encoded_labels);
    output.push(b'\n');
}

pub(super) fn begin_path_travel(
    output: &mut Vec<u8>,
    state: &mut EmitState,
    destination_feature: &str,
    travel_distance: f64,
) {
    // `GCode.cpp:7374-7392`: travel acceleration switches require the default
    // to be enabled and only apply a specific value when it is above zero.
    if state.options.default_acceleration == 0 {
        return;
    }
    let acceleration = if state.layer_index == 0 {
        (state.options.initial_layer_travel_acceleration > 0)
            .then_some(state.options.initial_layer_travel_acceleration)
    } else if travel_distance < state.options.retraction_minimum_travel {
        match destination_feature {
            "Overhang wall" => {
                (state.options.bridge_acceleration > 0).then_some(state.options.bridge_acceleration)
            }
            "Outer wall" => (state.options.outer_wall_acceleration > 0)
                .then_some(state.options.outer_wall_acceleration),
            _ => None,
        }
        .or((state.options.travel_acceleration > 0).then_some(state.options.travel_acceleration))
    } else {
        (state.options.travel_acceleration > 0).then_some(state.options.travel_acceleration)
    };
    set_acceleration(output, state, acceleration.unwrap_or(0));
}

pub(super) fn end_layer_for_timelapse(output: &mut Vec<u8>, state: &mut EmitState) {
    if state.options.retract_when_changing_layer && state.positioned {
        travel::retract_for_timelapse(output, state);
    }
}

fn set_acceleration(output: &mut Vec<u8>, state: &mut EmitState, acceleration: u32) {
    // `GCodeWriter.cpp:228`: zero means "keep the configured acceleration";
    // it neither emits nor updates the cached value.
    if acceleration == 0 || state.last_acceleration == Some(acceleration) {
        return;
    }
    output.extend_from_slice(format!("M204 S{acceleration}\n").as_bytes());
    state.last_acceleration = Some(acceleration);
}

pub(super) fn emit_layer(
    output: &mut Vec<u8>,
    layer: &mut OrderedExtrusionLayer,
    geometry: LayerGeometry<'_>,
    state: &mut EmitState,
) -> Result<(), SliceError> {
    for island in &mut layer.islands {
        let mut entities = std::mem::take(&mut island.entities);
        let infill_first = matches!(
            entities.first(),
            Some(
                IslandPrintEntity::Fill(_)
                    | IslandPrintEntity::FillCollection(_)
                    | IslandPrintEntity::Thin(_)
            )
        );
        if infill_first {
            let split = entities
                .iter()
                .position(|entity| matches!(entity, IslandPrintEntity::Perimeter(_)))
                .unwrap_or(entities.len());
            let perimeters = entities.split_off(split);
            emit_infills(output, &mut entities, geometry, state);
            for perimeter in perimeters {
                emit_perimeter(output, perimeter, geometry, state);
            }
        } else {
            let split = entities
                .iter()
                .position(|entity| !matches!(entity, IslandPrintEntity::Perimeter(_)))
                .unwrap_or(entities.len());
            for perimeter in entities.drain(..split) {
                emit_perimeter(output, perimeter, geometry, state);
            }
            emit_infills(output, &mut entities, geometry, state);
        }
    }
    Ok(())
}

fn emit_perimeter(
    output: &mut Vec<u8>,
    entity: IslandPrintEntity,
    geometry: LayerGeometry<'_>,
    state: &mut EmitState,
) {
    let IslandPrintEntity::Perimeter(collection) = entity else {
        unreachable!("perimeter phase contains only perimeter entities");
    };
    for loop_ in &collection.entities {
        loop_paths::emit(output, &loop_.extrusion_loop.paths, geometry, state);
    }
}

fn emit_infills(
    output: &mut Vec<u8>,
    entities: &mut Vec<IslandPrintEntity>,
    geometry: LayerGeometry<'_>,
    state: &mut EmitState,
) {
    if entities.is_empty() {
        return;
    }
    chain_and_reorder_entities(entities, local_cursor(state, geometry));
    for entity in entities.drain(..) {
        match entity {
            IslandPrintEntity::Fill(entity) => {
                emit_fill_entity(output, &entity, geometry, state);
            }
            IslandPrintEntity::FillCollection(collection) => {
                let collection = collection.chained_path_from(local_cursor(state, geometry));
                for entity in &collection.entities {
                    emit_fill_entity(output, entity, geometry, state);
                }
            }
            IslandPrintEntity::Thin(entity) => {
                emit_variable_width_entity(output, &entity, geometry, state);
            }
            IslandPrintEntity::Perimeter(_) => {
                unreachable!("infill phase contains only infill entities")
            }
        }
    }
}

fn emit_fill_entity(
    output: &mut Vec<u8>,
    entity: &FillExtrusionEntity,
    geometry: LayerGeometry<'_>,
    state: &mut EmitState,
) {
    match entity {
        FillExtrusionEntity::Path(path) => path::emit(
            output,
            path.polyline
                .points()
                .iter()
                .map(|point| (point.x(), point.y())),
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
                fitting: &path.fitting,
            },
            geometry,
            state,
        ),
        FillExtrusionEntity::VariableWidth(entity) => {
            emit_variable_width_entity(output, entity, geometry, state);
        }
    }
}

fn emit_variable_width_entity(
    output: &mut Vec<u8>,
    entity: &GapFillEntity,
    geometry: LayerGeometry<'_>,
    state: &mut EmitState,
) {
    match entity {
        GapFillEntity::Path(path) => emit_materialized_path(output, path, 0.0, geometry, state),
        GapFillEntity::Loop(paths) => loop_paths::emit(output, paths, geometry, state),
    }
}

fn local_cursor(state: &EmitState, geometry: LayerGeometry<'_>) -> Point {
    Point::new(
        geometry
            .scale
            .checked_scale(state.x - state.offset.0)
            .expect("emitted X remains in the coordinate domain"),
        geometry
            .scale
            .checked_scale(state.y - state.offset.1)
            .expect("emitted Y remains in the coordinate domain"),
    )
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
        ExtrusionRole::SolidInfill => "Internal solid infill",
    };
    path::emit(
        output,
        path.polyline.points.iter().map(|point| (point.x, point.y)),
        PathProperties {
            mm3_per_mm: path.mm3_per_mm,
            width: path.width,
            height: path.height,
            feature,
            is_perimeter: matches!(
                path.role,
                ExtrusionRole::ExternalPerimeter
                    | ExtrusionRole::Perimeter
                    | ExtrusionRole::OverhangPerimeter
            ),
            end_clip,
            fitting: &path.polyline.fitting,
        },
        geometry,
        state,
    );
}
