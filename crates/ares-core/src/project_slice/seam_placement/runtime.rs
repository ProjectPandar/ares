use super::{split_at, squared_distance};
use crate::{
    ProcessSeamPosition,
    geometry::CoordinateScale,
    project::effective_config::types::ResolvedProjectObject,
    project_slice::{
        island_print_order::{IslandPrintEntity, OrderedExtrusionLayer},
        perimeters::classic::{
            chained_loops::ExtrusionLoop,
            materialize::{ExtrusionRole, Point3},
            traversal::ClassicTraversalRecord,
        },
    },
};

pub(super) fn placement_modes(
    objects: &[ResolvedProjectObject],
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
            _ => None,
        })
        .collect()
}

pub(super) fn layer_mid_zs(records: &[Option<ClassicTraversalRecord>]) -> Vec<f32> {
    records
        .iter()
        .scan(0.0_f64, |print_z, record| {
            let height = record.as_ref().map_or(0.0, |record| record.layer_height);
            *print_z += height;
            Some((*print_z - 0.5 * height) as f32)
        })
        .collect()
}

pub(super) fn stagger_inner_seams(
    layers: &mut [OrderedExtrusionLayer],
    enabled: bool,
    scale: CoordinateScale,
) {
    if !enabled {
        return;
    }
    for loop_ in layers
        .iter_mut()
        .flat_map(|layer| &mut layer.islands)
        .flat_map(|island| &mut island.entities)
        .filter_map(|entity| match entity {
            IslandPrintEntity::Perimeter(collection) => Some(collection),
            _ => None,
        })
        .flat_map(|collection| &mut collection.entities)
        .map(|entity| &mut entity.extrusion_loop)
    {
        if let Some(target) = stagger_target(loop_, scale) {
            split_at(loop_, target, scale);
        }
    }
}

fn stagger_target(loop_: &ExtrusionLoop, scale: CoordinateScale) -> Option<(i64, i64)> {
    let first = loop_.paths.first()?;
    if first.role != ExtrusionRole::Perimeter {
        return None;
    }
    let mut remaining = f64::from(first.width) / scale.factor();
    for segment in loop_
        .paths
        .iter()
        .flat_map(|path| path.polyline.points.windows(2))
    {
        let dx = (segment[1].x - segment[0].x) as f64;
        let dy = (segment[1].y - segment[0].y) as f64;
        let length = dx.hypot(dy);
        if remaining <= length {
            let ratio = remaining / length;
            return Some((
                (segment[0].x as f64 + dx * ratio) as i64,
                (segment[0].y as f64 + dy * ratio) as i64,
            ));
        }
        remaining -= length;
    }
    None
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
        .map(|entity| &entity.extrusion_loop)
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
    split_at(loop_, (cursor.x, cursor.y), scale);
}
