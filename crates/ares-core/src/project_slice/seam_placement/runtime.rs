mod nearest;

pub(in crate::project_slice) use nearest::place_nearest_penalized;

use super::split::{split_at, split_at_projected, squared_distance};
use crate::{
    ProcessSeamPosition,
    geometry::CoordinateScale,
    project::effective_config::types::ResolvedProjectObject,
    project_slice::perimeters::classic::{chained_loops::ExtrusionLoop, materialize::Point3},
};

pub(super) fn placement_modes(
    objects: &[&ResolvedProjectObject],
    spiral_mode: bool,
) -> Vec<Option<ProcessSeamPosition>> {
    // GCode.cpp:8542-8549 bypasses SeamPlacer even below the spiral start layer.
    if spiral_mode {
        return vec![None; objects.len()];
    }
    objects
        .iter()
        .map(|object| Some(object.object.seam_position))
        .collect()
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
