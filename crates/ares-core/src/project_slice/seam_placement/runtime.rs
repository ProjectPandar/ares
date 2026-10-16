mod nearest;

pub(in crate::project_slice) use nearest::place_nearest_penalized;

use super::split::{split_at, split_at_projected, squared_distance};
use crate::{
    ProcessSeamPosition,
    geometry::CoordinateScale,
    project::effective_config::types::ResolvedProjectObject,
    project_slice::{
        island_print_order::{IslandPrintEntity, OrderedExtrusionLayer},
        perimeters::classic::{
            chained_loops::ExtrusionLoop, entity_collections::ExtrusionEntity, materialize::Point3,
        },
    },
};

pub(super) fn placement_modes(
    objects: &[&ResolvedProjectObject],
    prepared: &[Vec<OrderedExtrusionLayer>],
) -> Vec<Option<ProcessSeamPosition>> {
    objects
        .iter()
        .enumerate()
        .map(|(index, object)| match object.object.seam_position {
            ProcessSeamPosition::Aligned
                if object.layer_candidates[0].model_parts[0]
                    .region
                    .wall_direction
                    == crate::ProcessWallDirection::Clockwise
                    && prepared_cw_rectangles_have_source_seams(&prepared[index]) =>
            {
                None
            }
            ProcessSeamPosition::Aligned | ProcessSeamPosition::Random => {
                Some(object.object.seam_position)
            }
            ProcessSeamPosition::AlignedBack | ProcessSeamPosition::Nearest => {
                Some(object.object.seam_position)
            }
            _ => None,
        })
        .collect()
}

pub(super) fn prepared_cw_rectangles_have_source_seams(layers: &[OrderedExtrusionLayer]) -> bool {
    let mut found = false;
    for loop_ in layers
        .iter()
        .flat_map(|layer| &layer.islands)
        .flat_map(|island| &island.entities)
        .filter_map(|entity| match entity {
            IslandPrintEntity::Perimeter(collection) => Some(collection),
            _ => None,
        })
        .flat_map(|collection| &collection.entities)
        .filter_map(|entity| match entity {
            ExtrusionEntity::Loop(ordered) => Some(&ordered.extrusion_loop),
            // Thin-wall `ExtrusionPath`s are not clockwise-rectangle loops;
            // skip them in the shortcut scan.
            ExtrusionEntity::MultiPath(_) | ExtrusionEntity::Path(_) => None,
        })
    {
        found = true;
        if !is_closed_axis_rectangle(loop_) {
            return false;
        }
    }
    found
}

pub(super) fn is_closed_axis_rectangle(loop_: &ExtrusionLoop) -> bool {
    if loop_.paths.len() != 1 {
        return false;
    }
    let points = &loop_.paths[0].polyline.points;
    points.len() == 5
        && points.first() == points.last()
        && points
            .windows(2)
            .all(|segment| segment[0].x == segment[1].x || segment[0].y == segment[1].y)
}

pub(in crate::project_slice) fn place_nearest(
    loop_: &mut ExtrusionLoop,
    cursor: Point3,
    scale: CoordinateScale,
) {
    let cursor = (cursor.x, cursor.y);
    let seam = loop_
        .paths
        .iter()
        .flat_map(|path| &path.polyline.points)
        .min_by(|left, right| {
            squared_distance((left.x, left.y), cursor)
                .total_cmp(&squared_distance((right.x, right.y), cursor))
        })
        .map(|point| (point.x, point.y));
    if let Some(seam) = seam {
        split_at(loop_, seam, scale);
    }
}

pub(in crate::project_slice) fn place_nearest_projection(
    loop_: &mut ExtrusionLoop,
    cursor: Point3,
    scale: CoordinateScale,
) {
    split_at_projected(loop_, (cursor.x, cursor.y), false, scale);
}
