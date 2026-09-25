use super::split::{split_at, squared_distance};
use crate::{
    ProcessSeamPosition,
    geometry::CoordinateScale,
    project::effective_config::types::ResolvedProjectObject,
    project_slice::{
        island_print_order::{IslandPrintEntity, OrderedExtrusionLayer},
        perimeters::classic::{
            chained_loops::ExtrusionLoop,
            entity_collections::ExtrusionEntity,
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
            ProcessSeamPosition::AlignedBack | ProcessSeamPosition::Nearest => {
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
        .filter_map(|entity| match entity {
            // Staggered inner-wall seam splitting targets loops; open
            // thin-wall `ExtrusionPath`s are skipped (no seam). Arachne wall
            // generation stays typed-rejected before materialization, so no
            // multi-path can reach staggering.
            ExtrusionEntity::Loop(ordered) => Some(&mut ordered.extrusion_loop),
            ExtrusionEntity::MultiPath(_) | ExtrusionEntity::Path(_) => None,
        })
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

/// Penalty-aware Nearest seam selection (`SeamPlacer.cpp:1500-1560`):
/// geometrically match the loop to its perimeter via the closest
/// candidate, then minimize
/// `overhang + visibility + 1.0 * angle_penalty + distance_penalty`
/// over that perimeter's candidates
/// (`pick_nearest_seam_point_index` cpp:930-940), and split at the loop
/// vertex nearest the winning candidate.
pub(in crate::project_slice) fn place_nearest_penalized(
    loop_: &mut ExtrusionLoop,
    cursor: Point3,
    layer: &crate::project_slice::island_print_order::NearestSeamLayer,
    staggered: bool,
    scale: CoordinateScale,
) {
    let Some(first) = loop_
        .paths
        .first()
        .and_then(|path| path.polyline.points.first())
    else {
        return;
    };
    // `SeamPlacer.cpp:1527-1544`: match the loop to a perimeter by walking
    // consecutive loop points until two in a row resolve to the SAME
    // perimeter (T-junction disambiguation). Using only the first point can
    // latch onto an adjacent perimeter's candidates.
    let closest_perimeter = {
        let points_count: usize = loop_
            .paths
            .iter()
            .map(|path| path.polyline.points.len())
            .sum();
        let mut current = super::split::Projection {
            path: 0,
            segment: 0,
            x: first.x,
            y: first.y,
        };
        let mut perimeter = usize::MAX;
        for _ in 0..points_count {
            let query = (
                scale.unscale(current.x) as f32,
                scale.unscale(current.y) as f32,
            );
            let mut closest = usize::MAX;
            let mut closest_distance = f32::INFINITY;
            for (index, &(x, y)) in layer.positions.iter().enumerate() {
                let distance = (x - query.0).mul_add(x - query.0, (y - query.1) * (y - query.1));
                if distance < closest_distance {
                    closest_distance = distance;
                    closest = index;
                }
            }
            if closest == usize::MAX {
                return;
            }
            let candidate_perimeter = layer.perimeter_of_candidate[closest];
            if perimeter == candidate_perimeter {
                break;
            }
            perimeter = candidate_perimeter;
            current = super::split::next_loop_point(loop_, &current);
        }
        perimeter
    };
    let Some(&(start, end)) = layer.perimeter_ranges.get(closest_perimeter) else {
        return;
    };
    let cursor = (
        scale.unscale(cursor.x) as f32,
        scale.unscale(cursor.y) as f32,
    );
    if let Ok(dbg) = std::env::var("ARES_DUMP_SCORE") {
        use std::io::Write;
        if let Ok(mut f) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(dbg)
        {
            let _ = writeln!(f, "RANGE {start}..{end} cur {},{}", cursor.0, cursor.1);
            for index in start..end {
                let (x, y) = layer.positions[index];
                let _ = writeln!(
                    f,
                    "ACAND {index} {x:.6},{y:.6} oh={:.4} sc={:.4} emb={:.3} ang={:.4}",
                    layer.overhangs[index],
                    layer.scores[index],
                    layer.embedded[index],
                    layer.ccw_angles[index]
                );
            }
        }
    }
    // `SeamPlacer.cpp:760-794` `SeamComparator::is_first_better` (spNearest):
    // lexicographic cascade — overhang wins outright, then embedded
    // (hidden) points, then the composite penalty.
    let is_first_better = |a: usize, b: usize| -> bool {
        let overhang_a = layer.overhangs[a];
        let overhang_b = layer.overhangs[b];
        if overhang_a > 0.0 || overhang_b > 0.0 {
            return overhang_a < overhang_b;
        }
        if layer.embedded[a] < -0.5 && layer.embedded[b] > -0.5 {
            return true;
        }
        if layer.embedded[b] < -0.5 && layer.embedded[a] > -0.5 {
            return false;
        }
        let (ax, ay) = layer.positions[a];
        let (bx, by) = layer.positions[b];
        let dist_a = ((ax - cursor.0) * (ax - cursor.0) + (ay - cursor.1) * (ay - cursor.1)).sqrt();
        let dist_b = ((bx - cursor.0) * (bx - cursor.0) + (by - cursor.1) * (by - cursor.1)).sqrt();
        let distance_penalty_a =
            1.0 - crate::project_slice::seam_placement::gauss_penalty(dist_a, 0.005);
        let distance_penalty_b =
            1.0 - crate::project_slice::seam_placement::gauss_penalty(dist_b, 0.005);
        let penalty_a = overhang_a + layer.scores[a] + distance_penalty_a;
        let penalty_b = overhang_b + layer.scores[b] + distance_penalty_b;
        penalty_a < penalty_b
    };
    let mut best = usize::MAX;
    for index in start..end {
        if best == usize::MAX || is_first_better(index, best) {
            best = index;
        }
    }
    let (seam_x, seam_y) = layer.positions[best];
    let (Some(seam_x), Some(seam_y)) = (
        scale.checked_scale(seam_x as f64),
        scale.checked_scale(seam_y as f64),
    ) else {
        return;
    };
    let seam_query = (seam_x, seam_y);
    // External loops split directly at the candidate; inner loops project
    // the candidate onto the loop and stagger it (`SeamPlacer.cpp:1562-1618`).
    if loop_
        .paths
        .iter()
        .all(|path| path.role != ExtrusionRole::ExternalPerimeter)
    {
        // `get_closest_path_and_point(seam_point, false)`
        // (`SeamPlacer.cpp:1567`) — no overhang preference for the inner
        // projection; the final `split_at` prefers non-overhang itself.
        let projection = super::split::closest_projection(&loop_.paths, seam_query, false);
        let depth = squared_distance((projection.x, projection.y), seam_query).sqrt();
        let angle = layer.ccw_angles[best];
        let beta = (angle / 2.0).cos();
        let depth = if beta.abs() > f32::EPSILON {
            depth * f64::from(beta) / std::f64::consts::SQRT_2
        } else {
            depth
        };
        // `SeamPlacer.cpp:1602` — the stagger depth is at least the path
        // width (it is sometimes strongly underestimated).
        let width = f64::from(loop_.paths[projection.path].width) / scale.factor();
        let depth = depth.max(width);
        let target = if staggered {
            walk_along(loop_, &projection, depth)
        } else {
            (projection.x, projection.y)
        };
        split_at(loop_, target, scale);
        return;
    }
    let seam = loop_
        .paths
        .iter()
        .flat_map(|path| &path.polyline.points)
        .min_by(|left, right| {
            squared_distance((left.x, left.y), seam_query)
                .total_cmp(&squared_distance((right.x, right.y), seam_query))
        })
        .map(|point| (point.x, point.y));
    if let Some(seam) = seam {
        split_at(loop_, seam, scale);
    }
}

/// Walk `distance` (scaled units) forward along the loop from `from`,
/// wrapping around (`SeamPlacer.cpp:1601-1617`).
fn walk_along(loop_: &ExtrusionLoop, from: &super::split::Projection, distance: f64) -> (i64, i64) {
    let mut remaining = distance;
    let mut position = (from.x, from.y);
    let mut started = false;
    for pass in 0..2 {
        for (path_index, path) in loop_.paths.iter().enumerate() {
            for (segment_index, segment) in path.polyline.points.windows(2).enumerate() {
                if !started {
                    if path_index == from.path && segment_index == from.segment {
                        started = true;
                    } else {
                        continue;
                    }
                }
                let end = (segment[1].x, segment[1].y);
                let length = squared_distance(position, end).sqrt();
                if remaining <= length {
                    let ratio = if length == 0.0 {
                        0.0
                    } else {
                        remaining / length
                    };
                    return (
                        (position.0 as f64 + (end.0 - position.0) as f64 * ratio) as i64,
                        (position.1 as f64 + (end.1 - position.1) as f64 * ratio) as i64,
                    );
                }
                remaining -= length;
                position = end;
            }
        }
        if pass == 1 {
            break;
        }
    }
    position
}

pub(in crate::project_slice) fn place_nearest_projection(
    loop_: &mut ExtrusionLoop,
    cursor: Point3,
    scale: CoordinateScale,
) {
    split_at(loop_, (cursor.x, cursor.y), scale);
}
