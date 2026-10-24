// Perimeter emission from `GCode::extrude_perimeters`
// (`GCode.cpp:6131-6147`): every collection entity dispatches through
// `GCode::extrude_entity` (`GCode.cpp:6091`).

mod multi_path;

use super::{local_cursor, loop_paths, state::EmitState, state::LayerGeometry};
use crate::project_slice::{
    island_print_order::IslandPrintEntity, perimeters::classic::entity_collections::ExtrusionEntity,
};

pub(super) fn emit_perimeter(
    output: &mut Vec<u8>,
    entity: IslandPrintEntity,
    geometry: LayerGeometry<'_>,
    state: &mut EmitState,
) -> Result<(), crate::SliceError> {
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
            // Thin-wall `ExtrusionPath`s participate in the wipe-touch
            // touching-lines gate like any other perimeter entity.
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
    for entity in collection.entities {
        let mut loop_ = match entity {
            ExtrusionEntity::Loop(loop_) => loop_,
            // `GCode::extrude_path` (`GCode.cpp:6092`) — a bare `ExtrusionPath`
            // (thin-wall `variable_width` materialization) emits directly with
            // no loop clipping/seam step.
            ExtrusionEntity::Path(path) => {
                super::materialized::emit(
                    output,
                    state,
                    super::materialized::Emission {
                        path: &path,
                        end_clip: 0.0,
                        slope: None,
                        geometry,
                        inside_loop: false,
                    },
                )?;
                continue;
            }
            ExtrusionEntity::MultiPath(multi_path) => {
                multi_path::emit(output, &multi_path.paths, geometry, state)?;
                continue;
            }
        };
        // `estimate_malformations` (`PrintObject.cpp:900-914`) runs BEFORE
        // seam placement, so its `collect_points` sees the loop's natural
        // segmentation — no seam vertex splitting a mid-edge segment and no
        // fragment boundary truncating the 8mm curvature `walk`. Collect
        // here on the untouched loop; the per-path `estimate` call then
        // skips its own collect via `inside_loop`.
        if state.options.enable_overhang_speed {
            state
                .curl
                .collect_entity(state.layer_index, &loop_.extrusion_loop.paths, &geometry);
        }
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
            &region_perimeters,
        )?;
    }
    Ok(())
}
