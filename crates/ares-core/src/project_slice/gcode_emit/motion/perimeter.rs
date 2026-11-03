//! `GCode.cpp:8889-8933`, region-scoped perimeter passes and printed wipe support.

mod multi_path;

use super::{
    local_cursor, loop_paths, loop_paths::wipe_inward::WipeInwardSupport, state::EmitState,
    state::LayerGeometry,
};
use crate::project_slice::{
    island_print_order::IslandPrintEntity, perimeters::classic::entity_collections::ExtrusionEntity,
};

pub(super) fn emit_perimeter(
    output: &mut Vec<u8>,
    (entity, defer_unsupported, unsupported_loops_only, mut wipe_support): (
        &IslandPrintEntity,
        bool,
        bool,
        Option<&mut WipeInwardSupport>,
    ),
    geometry: LayerGeometry<'_>,
    state: &mut EmitState,
) -> Result<(), crate::SliceError> {
    let IslandPrintEntity::Perimeter(collection) = entity else {
        unreachable!("perimeter phase contains only perimeter entities");
    };
    let selected = |entity: &ExtrusionEntity| {
        !defer_unsupported || waits_for_infill(entity) == unsupported_loops_only
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
        emit_entity(
            output,
            (entity, wipe_support.as_deref()),
            geometry,
            state,
            &region_perimeters,
        )?;
        if let Some(support) = wipe_support.as_deref_mut() {
            support.append(entity);
        }
    }
    Ok(())
}

pub(super) fn preseed_wipe_support(entity: &IslandPrintEntity, support: &mut WipeInwardSupport) {
    let IslandPrintEntity::Perimeter(collection) = entity else {
        unreachable!("perimeter phase contains only perimeter entities");
    };
    for entity in collection
        .entities
        .iter()
        .filter(|entity| !waits_for_infill(entity))
    {
        support.append(entity);
    }
}

fn waits_for_infill(entity: &ExtrusionEntity) -> bool {
    matches!(entity, ExtrusionEntity::Loop(loop_) if loop_.extrusion_loop.print_after_infill)
}

fn emit_entity(
    output: &mut Vec<u8>,
    (entity, wipe_support): (&ExtrusionEntity, Option<&WipeInwardSupport>),
    geometry: LayerGeometry<'_>,
    state: &mut EmitState,
    region_perimeters: &[Vec<(i64, i64)>],
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
        (
            &loop_.extrusion_loop,
            loop_.unsupported_distance,
            wipe_support,
        ),
        geometry,
        state,
        region_perimeters,
    )
}
