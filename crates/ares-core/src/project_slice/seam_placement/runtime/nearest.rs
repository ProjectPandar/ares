use super::super::{
    kdtree,
    split::{self, split_at},
};
use crate::{
    geometry::CoordinateScale,
    project_slice::perimeters::classic::{
        chained_loops::ExtrusionLoop,
        materialize::{ExtrusionRole, Point3},
    },
};

#[expect(
    clippy::approx_constant,
    reason = "OrcaSlicer uses the literal 1.4142 seam-depth coefficient"
)]
pub(in crate::project_slice) fn place_nearest_penalized(
    loop_: &mut ExtrusionLoop,
    cursor: Point3,
    layer: &crate::project_slice::island_print_order::NearestSeamLayer,
    staggered: bool,
    scale: CoordinateScale,
) -> Option<f32> {
    let first = loop_
        .paths
        .first()
        .and_then(|path| path.polyline.points.first())?;
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
        let mut current = split::Projection {
            path: 0,
            segment: 0,
            x: first.x,
            y: first.y,
        };
        let mut perimeter = usize::MAX;
        // Upstream queries a `KDTreeIndirect<3, float>` built over the
        // layer's candidate `Vec3f` positions (`SeamPlacer.cpp:1033-1036`,
        // `:1536`); equal-distance ties resolve to the last VISITED node,
        // so the balanced-build/visit order is part of the observable
        // result — port it verbatim.
        let coordinate = |index: usize, dim: usize| -> f32 {
            let p = layer.positions[index];
            [p.0, p.1, p.2][dim]
        };
        // Upstream builds the tree once per layer (`SeamPlacer.cpp:
        // 1021-1036`); cache it on the layer's seam data.
        let tree = layer
            .kdtree
            .get_or_init(|| kdtree::KdTree::build(layer.positions.len(), coordinate));
        // `to_3d(unscaled_p, unscaled_z)` (`SeamPlacer.cpp:1537`): the
        // query z is the same constant `slice_z` every candidate carries.
        let query_z = layer.positions.first().map(|p| p.2).unwrap_or(0.0);
        for _ in 0..points_count {
            let query = [
                scale.unscale(current.x) as f32,
                scale.unscale(current.y) as f32,
                query_z,
            ];
            let closest = tree.find_closest(query, &coordinate)?;
            let candidate_perimeter = layer.perimeter_of_candidate[closest];
            if perimeter == candidate_perimeter {
                break;
            }
            perimeter = candidate_perimeter;
            current = split::next_loop_point(loop_, &current);
        }
        perimeter
    };
    let &(start, end) = layer.perimeter_ranges.get(closest_perimeter)?;
    let cursor = (
        scale.unscale(cursor.x) as f32,
        scale.unscale(cursor.y) as f32,
    );
    // SeamPlacer.cpp:1176–1222: paint outranks geometric context and distance.
    let is_first_better = |a: usize, b: usize| -> bool {
        if layer.paint_states[a] != layer.paint_states[b] {
            return layer.paint_states[a] > layer.paint_states[b];
        }
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
        let (ax, ay, _) = layer.positions[a];
        let (bx, by, _) = layer.positions[b];
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
    let unsupported_distance = layer
        .unsupported_distances
        .as_ref()
        .map(|values| values[best]);
    let (seam_x, seam_y, _) = layer.positions[best];
    let (Some(seam_x), Some(seam_y)) = (
        scale.checked_scale(seam_x as f64),
        scale.checked_scale(seam_y as f64),
    ) else {
        return None;
    };
    let seam_query = (seam_x, seam_y);
    // External loops split directly at the candidate; inner loops project
    // the candidate onto the loop and stagger it (`SeamPlacer.cpp:1564-1618`).
    // Upstream keys the inner-seam path on `loop.role()` — the FIRST
    // path's role (`ExtrusionEntity.hpp:485`) — not every path's role.
    if loop_
        .paths
        .first()
        .is_some_and(|path| path.role == ExtrusionRole::Perimeter)
    {
        // `get_closest_path_and_point(seam_point, false)`
        // (`SeamPlacer.cpp:1567`) — no overhang preference for the inner
        // projection; the final `split_at` prefers non-overhang itself.
        let mut projection = split::closest_projection(&loop_.paths, seam_query, false);
        // `depth` is the UNSCALED distance from the candidate to the
        // projected foot (`SeamPlacer.cpp:1569` works in mm on `Vec3f`).
        // `seam_position` is the unscaled candidate position (`Vec3f`,
        // mm); `depth` is `unscale(new_scale(seam_position) - foot_pt)` —
        // the seam coordinate is round-tripped through the scaled domain
        // (`SeamPlacer.cpp:1562-1569`). All floats are `f32` upstream.
        let seam_pos = layer.positions[best];
        // `unscale(Point(seam_point - foot_pt))` — the difference is
        // computed on SCALED ints, then `unscale` produces a `Vec2d`
        // (`double` divide by SCALE_FACTOR) whose `.norm()` is f64, stored
        // into `float depth` (`SeamPlacer.cpp:1568-1569`).
        let scale_factor_f64 = scale.factor();
        let dx_mm = (seam_x - projection.x) as f64 * scale_factor_f64;
        let dy_mm = (seam_y - projection.y) as f64 * scale_factor_f64;
        let mut depth = (dx_mm * dx_mm + dy_mm * dy_mm).sqrt() as f32;
        let angle = layer.ccw_angles[best];
        let beta = (angle / 2.0).cos();
        // `SeamPlacer.cpp:1571-1580`: wrap the candidate index inside its
        // perimeter range (start_index..end_index semantics).
        let (perimeter_start, perimeter_end) = layer.perimeter_ranges[closest_perimeter];
        let index_of_prev = if best == perimeter_start {
            perimeter_end - 1
        } else {
            best - 1
        };
        let index_of_next = if best == perimeter_end - 1 {
            perimeter_start
        } else {
            best + 1
        };
        let candidate = layer.positions[best];
        let cand_pos = (candidate.0, candidate.1);
        // Concave-angle overshoot (`SeamPlacer.cpp:1581-1593`): when the
        // outer seam sits on a concave corner, push the inner seam past
        // the perpendicular depth along `dir_to_middle` and re-project.
        // The gate is literally `squaredNorm() < depth` — upstream
        // compares a SQUARED distance against the un-squared depth.
        let dist_to_candidate_sq =
            (seam_pos.0 - cand_pos.0).powi(2) + (seam_pos.1 - cand_pos.1).powi(2);
        // `local_ccw_angle < -EPSILON` — upstream EPSILON is `1e-4`
        // (double literal, `libslic3r.h:52`), not `f32::EPSILON`.
        if dist_to_candidate_sq < depth && angle < -1e-4 {
            let prev = layer.positions[index_of_prev];
            let next = layer.positions[index_of_next];
            let to_prev = (cand_pos.0 - prev.0, cand_pos.1 - prev.1);
            let to_next = (cand_pos.0 - next.0, cand_pos.1 - next.1);
            let dir_to_middle = {
                let (px, py) = split::normalized(to_prev);
                let (nx, ny) = split::normalized(to_next);
                ((px + nx) * 0.5, (py + ny) * 0.5)
            };
            // `1.4142`/`1.4142` are double literals upstream — promote
            // depth to f64 for the arithmetic, narrow back to float.
            depth = (1.4142 * f64::from(depth) / f64::from(beta)) as f32;
            let final_pos = (
                cand_pos.0 + depth * dir_to_middle.0,
                cand_pos.1 + depth * dir_to_middle.1,
            );
            if let (Some(sx), Some(sy)) = (
                scale.checked_scale(f64::from(final_pos.0)),
                scale.checked_scale(f64::from(final_pos.1)),
            ) {
                projection = split::closest_projection(&loop_.paths, (sx, sy), false);
            }
        } else {
            // Convex corner (`SeamPlacer.cpp:1597-1598`): shrink the
            // center-line depth to the perpendicular depth.
            // `depth * beta_angle` stays f32 upstream; only the division
            // by the double literal `1.4142` widens.
            depth = (f64::from(depth * beta) / 1.4142) as f32;
        }
        let target = (projection.x, projection.y);
        // `SeamPlacer.cpp:1601-1617`: optional staggered-inner-seam walk —
        // `depth` is clamped up to the path width (mm) first, then walked
        // along the scaled loop coordinates.
        if staggered {
            // `std::max(width, depth)` then an unscaled-f32 walk
            // (`SeamPlacer.cpp:1603-1616`).
            let width = loop_.paths[projection.path].width;
            let depth_mm = depth.max(width);
            let target = walk_along(loop_, &projection, depth_mm, scale);
            split_at(loop_, target, scale);
            return unsupported_distance;
        }
        split_at(loop_, target, scale);
        return unsupported_distance;
    }
    // Non-`erPerimeter` loops (external/overhang): upstream passes the
    // scaled seam candidate straight into `split_at_vertex` with its
    // 1.5µm snap (`SeamPlacer.cpp:1620-1627`) — no vertex search.
    split_at(loop_, seam_query, scale);
    unsupported_distance
}

/// Walk `distance_mm` forward along the loop from `from`, wrapping around.
/// Upstream operates on unscaled `Vec2f` and subtracts each whole segment's
/// length even when it overshoots (`SeamPlacer.cpp:1604-1616`), so the
/// interpolation uses the f32 segment ratio before `new_scale` truncates.
fn walk_along(
    loop_: &ExtrusionLoop,
    from: &split::Projection,
    distance_mm: f32,
    scale: CoordinateScale,
) -> (i64, i64) {
    let mut remaining = distance_mm;
    let mut current = *from;
    // Upstream `unscale(...)` is `coord_t → Vec2d` (f64 multiply by
    // SCALING_FACTOR), then `cast<float>()` narrows — NOT an f32 multiply
    // (`SeamPlacer.cpp:1606-1607`).
    let unscale_f32 = |coordinate: i64| -> f32 { scale.unscale(coordinate) as f32 };
    let mut foot = (unscale_f32(current.x), unscale_f32(current.y));
    while remaining > 0.0 {
        let next = split::next_loop_point(loop_, &current);
        let target = (unscale_f32(next.x), unscale_f32(next.y));
        let distance = ((target.0 - foot.0).powi(2) + (target.1 - foot.1).powi(2)).sqrt();
        let mut landed = next;
        if distance > remaining {
            // `a + (b - a) * depth / dist` on `Vec2f`, then
            // `Point::new_scale` truncates (`SeamPlacer.cpp:1611-1614`).
            let x = foot.0 + (target.0 - foot.0) * remaining / distance;
            let y = foot.1 + (target.1 - foot.1) * remaining / distance;
            landed.x = (f64::from(x) / scale.factor()) as i64;
            landed.y = (f64::from(y) / scale.factor()) as i64;
        }
        remaining -= distance;
        current = landed;
        foot = (unscale_f32(current.x), unscale_f32(current.y));
    }
    (current.x, current.y)
}
