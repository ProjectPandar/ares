//! `GCode.cpp:8889-8933`, the `wipe_support == nullopt` perimeter passes.

mod multi_path;

use super::{local_cursor, loop_paths, state::EmitState, state::LayerGeometry};
use crate::project_slice::{
    island_print_order::IslandPrintEntity, perimeters::classic::entity_collections::ExtrusionEntity,
};

pub(super) fn emit_perimeter(
    output: &mut Vec<u8>,
    (entity, defer_unsupported, unsupported_loops_only): (&IslandPrintEntity, bool, bool),
    geometry: LayerGeometry<'_>,
    state: &mut EmitState,
) -> Result<(), crate::SliceError> {
    let IslandPrintEntity::Perimeter(collection) = entity else {
        unreachable!("perimeter phase contains only perimeter entities");
    };
    let selected = |entity: &ExtrusionEntity| {
        let waits_for_infill = matches!(entity, ExtrusionEntity::Loop(loop_)
            if loop_.extrusion_loop.print_after_infill);
        !defer_unsupported || waits_for_infill == unsupported_loops_only
    };
    if !collection.entities.iter().any(selected) {
        return Ok(());
    }
    // Keep the complete original collection for the existing touching-lines
    // gate, independent of which entities this pass emits.
    let region_perimeters: Vec<Vec<(i64, i64)>> = collection
        .entities
        .iter()
        .map(|entity| match entity {
            ExtrusionEntity::Loop(loop_) => loop_
                .extrusion_loop
                .paths
                .iter()
                .flat_map(|path| path.polyline.points.iter().map(|point| (point.x, point.y)))
                .collect(),
            ExtrusionEntity::Path(path) => path
                .polyline
                .points
                .iter()
                .map(|point| (point.x, point.y))
                .collect(),
            ExtrusionEntity::MultiPath(multi_path) => multi_path
                .paths
                .iter()
                .flat_map(|path| path.polyline.points.iter().map(|point| (point.x, point.y)))
                .collect(),
        })
        .collect();
    for entity in collection.entities.iter().filter(|entity| selected(entity)) {
        emit_entity(output, (entity, &region_perimeters), geometry, state)?;
    }
    Ok(())
}

fn emit_entity(
    output: &mut Vec<u8>,
    (entity, region_perimeters): (&ExtrusionEntity, &[Vec<(i64, i64)>]),
    geometry: LayerGeometry<'_>,
    state: &mut EmitState,
) -> Result<(), crate::SliceError> {
    let mut loop_ = match entity {
        ExtrusionEntity::Loop(loop_) => loop_.clone(),
        ExtrusionEntity::Path(path) => {
            return super::materialized::emit(
                output,
                state,
                super::materialized::Emission {
                    path,
                    end_clip: 0.0,
                    slope: None,
                    geometry,
                },
            );
        }
        ExtrusionEntity::MultiPath(multi_path) => {
            return multi_path::emit(output, &multi_path.paths, geometry, state);
        }
    };
    if state.spiral_vase {
        crate::project_slice::seam_placement::place_nearest_projection(
            &mut loop_.extrusion_loop,
            crate::project_slice::perimeters::classic::materialize::Point3 {
                x: local_cursor(state, geometry).x(),
                y: local_cursor(state, geometry).y(),
                z: 0,
            },
            geometry.scale,
        );
    } else if state.options.seam_position == crate::ProcessSeamPosition::Nearest {
        let cursor = crate::project_slice::perimeters::classic::materialize::Point3 {
            x: local_cursor(state, geometry).x(),
            y: local_cursor(state, geometry).y(),
            z: 0,
        };
        if let Some(layer) = geometry.nearest_seam_penalties {
            loop_.unsupported_distance =
                crate::project_slice::seam_placement::place_nearest_penalized(
                    &mut loop_.extrusion_loop,
                    cursor,
                    layer,
                    geometry.staggered_inner,
                    geometry.scale,
                );
        } else {
            crate::project_slice::seam_placement::place_nearest(
                &mut loop_.extrusion_loop,
                cursor,
                geometry.scale,
            );
        }
    }
    loop_paths::emit(
        output,
        (&loop_.extrusion_loop, loop_.unsupported_distance),
        geometry,
        state,
        region_perimeters,
    )
}
