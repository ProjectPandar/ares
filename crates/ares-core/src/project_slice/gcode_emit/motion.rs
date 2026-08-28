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

pub(in crate::project_slice) use arc::simplify_points;
pub(super) use state::{
    EmitState, LayerGeometry, append_exclude_end, append_object_start, begin_layer,
    begin_path_travel, queue_exclude_end, queue_exclude_start, queue_object_start,
};
pub(super) use travel::{flush_pending_retract_lift, flush_pending_retract_wipe};

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
    output.extend_from_slice(
        format!(
            "G1 E{} F{}\n",
            format::extrusion(-length),
            format::axis(state.options.retraction_feedrate)
        )
        .as_bytes(),
    );
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

fn set_acceleration(output: &mut Vec<u8>, state: &mut EmitState, acceleration: u32, travel: bool) {
    let separate_travel = travel
        && matches!(
            state.options.gcode_flavor,
            crate::GCodeFlavor::Repetier
                | crate::GCodeFlavor::MarlinFirmware
                | crate::GCodeFlavor::RepRapFirmware
        );
    let limit = if travel {
        state.options.max_travel_acceleration
    } else {
        state.options.max_acceleration
    };
    // `GCodeWriter.cpp:218-221`: clamp by the machine limit, then skip zero
    // or unchanged values without touching the cached one.
    let acceleration = if limit > 0 && acceleration > limit {
        limit
    } else {
        acceleration
    };
    let last = if separate_travel {
        &mut state.last_travel_acceleration
    } else {
        &mut state.last_acceleration
    };
    if acceleration == 0 || *last == Some(acceleration) {
        return;
    }
    let line = match state.options.gcode_flavor {
        crate::GCodeFlavor::Repetier => {
            let code = if separate_travel { "M202" } else { "M201" };
            format!("{code} X{acceleration} Y{acceleration}\n")
        }
        crate::GCodeFlavor::RepRapFirmware | crate::GCodeFlavor::MarlinFirmware => {
            let code = if separate_travel { "M204 T" } else { "M204 P" };
            format!("{code}{acceleration}\n")
        }
        _ => format!("M204 S{acceleration}\n"),
    };
    output.extend_from_slice(line.as_bytes());
    *last = Some(acceleration);
}

/// Klipper merges acceleration and jerk into one `SET_VELOCITY_LIMIT` line
/// (`GCodeWriter.cpp:324-348`, `GCode.cpp:7409-7412`); other flavors emit
/// separate acceleration and jerk commands.
pub(super) fn set_accel_and_jerk(
    output: &mut Vec<u8>,
    state: &mut EmitState,
    acceleration: u32,
    jerk: f64,
    travel: bool,
) {
    if state.options.gcode_flavor != crate::GCodeFlavor::Klipper {
        set_acceleration(output, state, acceleration, travel);
        jerk::set(output, state, jerk);
        return;
    }
    let acceleration =
        if state.options.max_acceleration > 0 && acceleration > state.options.max_acceleration {
            state.options.max_acceleration
        } else {
            acceleration
        };
    let jerk = jerk::clamp_xy(state, jerk);
    let mut line = String::from("SET_VELOCITY_LIMIT");
    let mut empty = true;
    if acceleration != 0 && state.last_acceleration != Some(acceleration) {
        line.push_str(&format!(" ACCEL={acceleration}"));
        if state.options.accel_to_decel_enable {
            // streams as a double with ostream's default 6-significant digits
            let decel = acceleration as f64 * state.options.accel_to_decel_factor / 100.0;
            line.push_str(&format!(" ACCEL_TO_DECEL={}", format::axis(decel)));
        }
        state.last_acceleration = Some(acceleration);
        empty = false;
    }
    if jerk > 0.01
        && !state
            .last_jerk
            .is_some_and(|last| (last - jerk).abs() < 1.0e-6)
    {
        line.push_str(&format!(" SQUARE_CORNER_VELOCITY={}", format::axis(jerk)));
        state.last_jerk = Some(jerk);
        empty = false;
    }
    if !empty {
        line.push('\n');
        output.extend_from_slice(line.as_bytes());
    }
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
