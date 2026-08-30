mod acceleration;
mod arc;
mod clip;
mod extrusion;
mod fan;
mod features;
mod format;
mod jerk;
mod loop_paths;
mod options;
mod overhang;
mod path;
#[cfg(test)]
#[path = "motion/path/tests.rs"]
mod path_tests;
mod state;
#[cfg(test)]
mod tests;
mod travel;

pub(super) use acceleration::{set_accel_and_jerk, set_layer_acceleration_and_jerk};
pub(in crate::project_slice) use arc::simplify_points;
pub(super) use state::{
    EmitState, LayerGeometry, LiftMode, append_exclude_end, append_object_start, begin_layer,
    begin_path_travel, queue_exclude_end, queue_exclude_start, queue_object_start,
    queue_object_stop_label,
};
pub(super) use travel::{
    flush_pending_retract_lift, flush_pending_retract_wipe, retract_for_print_end,
};

use features::PathProperties;
pub(in crate::project_slice::gcode_emit) use options::{MotionOptions, first_nullable_float};

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

pub(super) fn prepare_traditional_timelapse(output: &mut Vec<u8>, state: &mut EmitState) {
    travel::retract_for_timelapse(output, state);
}

pub(super) fn defer_layer_retraction(state: &mut EmitState) {
    if state.options.retract_when_changing_layer && state.positioned {
        state.pending_layer_retract = true;
    }
}

pub(super) fn end_layer_for_timelapse(output: &mut Vec<u8>, state: &mut EmitState) {
    if state.options.retract_when_changing_layer && state.positioned {
        if state.tags.is_bbl() {
            travel::retract_for_timelapse(output, state);
        } else {
            // Compatible flavor defers the retraction past the layer
            // marker block; `flush_pending_retract_wipe`/`_lift` emit it.
            state.pending_layer_retract = true;
        }
    }
}

/// Configured retraction at the start of the first layer — matches the
/// GCodeWriter `retract()` formatting (`GCodeWriter.cpp`).
pub(super) fn retract_before_layer(output: &mut Vec<u8>, state: &mut EmitState) {
    let length = state.options.retraction_length;
    if length <= 0.0 {
        return;
    }
    let retract = extrusion::coordinate(state, -length);
    output.extend_from_slice(
        format!(
            "G1 E{} F{}\n",
            format::extrusion(retract),
            format::axis(state.options.retraction_feedrate)
        )
        .as_bytes(),
    );
    if !state.options.use_relative_e_distances {
        output.extend_from_slice(b"G92 E0\n");
        state.e_position = 0.0;
    }
    state.current_feedrate = state.options.retraction_feedrate;
    state.retracted = true;
}

#[derive(Clone, Copy)]
pub(super) struct SkirtLoopFlow {
    pub(super) width: f32,
    pub(super) height: f32,
    pub(super) mm3_per_mm: f64,
}

pub(super) fn emit_brim_loop(
    output: &mut Vec<u8>,
    points: impl Iterator<Item = (i64, i64)>,
    flow: SkirtLoopFlow,
    geometry: LayerGeometry<'_>,
    state: &mut EmitState,
) {
    path::emit(
        output,
        points,
        PathProperties {
            mm3_per_mm: flow.mm3_per_mm,
            width: flow.width,
            height: flow.height,
            feature: "Brim",
            is_perimeter: false,
            end_clip: state.options.seam_gap,
            fitting: &[],
        },
        geometry,
        state,
    );
}

pub(super) fn emit_skirt_loop(
    output: &mut Vec<u8>,
    points: impl Iterator<Item = (i64, i64)>,
    flow: SkirtLoopFlow,
    geometry: LayerGeometry<'_>,
    state: &mut EmitState,
) {
    path::emit(
        output,
        points,
        PathProperties {
            mm3_per_mm: flow.mm3_per_mm,
            width: flow.width,
            height: flow.height,
            feature: "Skirt",
            is_perimeter: false,
            // `GCode::extrude_loop` clips every loop by the seam gap
            // (`GCode.cpp:5778-5790`).
            end_clip: state.options.seam_gap,
            fitting: &[],
        },
        geometry,
        state,
    );
    // `GCode.cpp:5979-5991` stores loop wipe paths forward so the wipe
    // wraps from the loop end back toward the path start.
    state.wipe_path.reverse();
}

pub(super) fn emit_layer<F>(
    output: &mut Vec<u8>,
    layer: &mut OrderedExtrusionLayer,
    geometry: LayerGeometry<'_>,
    state: &mut EmitState,
    mut before_first_infill: F,
) -> Result<bool, SliceError>
where
    F: FnMut(&mut Vec<u8>, &mut EmitState) -> Result<bool, SliceError>,
{
    let mut interlude_emitted = false;
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
            if !interlude_emitted && !entities.is_empty() {
                interlude_emitted = before_first_infill(output, state)?;
            }
            emit_infills(output, &mut entities, geometry, state);
        }
    }
    Ok(interlude_emitted)
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
    for mut loop_ in collection.entities {
        if state.options.seam_position == crate::ProcessSeamPosition::Nearest {
            crate::project_slice::seam_placement::place_nearest(
                &mut loop_.extrusion_loop,
                crate::project_slice::perimeters::classic::materialize::Point3 {
                    x: local_cursor(state, geometry).x(),
                    y: local_cursor(state, geometry).y(),
                    z: 0,
                },
                geometry.scale,
            );
        }
        loop_paths::emit(
            output,
            &loop_.extrusion_loop.paths,
            loop_.extrusion_loop.role,
            geometry,
            state,
        );
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
        GapFillEntity::Loop(paths) => loop_paths::emit(
            output,
            paths,
            crate::project_slice::perimeters::classic::chained_loops::ExtrusionLoopRole::Internal,
            geometry,
            state,
        ),
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
        ExtrusionRole::TopSolidInfill => "Top surface",
        ExtrusionRole::BottomSurface => "Bottom surface",
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
