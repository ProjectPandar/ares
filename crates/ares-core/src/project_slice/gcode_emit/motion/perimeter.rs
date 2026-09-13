// Perimeter emission from `GCode::extrude_perimeters`
// (`GCode.cpp:6131-6147`): every collection entity dispatches through
// `GCode::extrude_entity` (`GCode.cpp:6091`).

use super::{local_cursor, loop_paths, state::EmitState, state::LayerGeometry};
use crate::project_slice::{
    island_print_order::IslandPrintEntity,
    perimeters::classic::{chained_loops::ExtrusionLoopRole, entity_collections::ExtrusionEntity},
};

pub(super) fn emit_perimeter(
    output: &mut Vec<u8>,
    entity: IslandPrintEntity,
    geometry: LayerGeometry<'_>,
    state: &mut EmitState,
) {
    let IslandPrintEntity::Perimeter(collection) = entity else {
        unreachable!("perimeter phase contains only perimeter entities");
    };
    // `region.perimeters` feeds the wipe-before-external hop's
    // touching-lines gate (`GCode.cpp:6144, 5824, 5871-5882`): each
    // entity counts once when any of its polyline segments lies within
    // one nozzle diameter of the de-retraction point.
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
            _ => Vec::new(),
        })
        .collect();
    for entity in collection.entities {
        let ExtrusionEntity::Loop(mut loop_) = entity else {
            // `GCode::extrude_multi_path` (`GCode.cpp:6038`) is not ported yet;
            // arachne wall generation stays typed-rejected before
            // materialization, so no multi-path can reach emission.
            unreachable!(
                "multi-path perimeter emission lands with the arachne materialization seam"
            );
        };
        if state.spiral_vase && loop_.extrusion_loop.role != ExtrusionLoopRole::Hole {
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
            &loop_.extrusion_loop.paths,
            loop_.extrusion_loop.role,
            geometry,
            state,
            &region_perimeters,
        );
    }
}
