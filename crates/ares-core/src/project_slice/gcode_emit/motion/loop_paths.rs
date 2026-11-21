pub(super) mod adaptive_pa;
mod inward;
mod small_perimeter;
pub(super) mod wipe_inward;
use inward::{ccw_angle, loop_is_counter_clockwise, path_length};

use super::{EmitState, LayerGeometry, format::axis as format_axis};
use crate::project_slice::perimeters::classic::{
    chained_loops::ExtrusionLoopRole,
    materialize::{ExtrusionPath, ExtrusionRole, Point3},
};

pub(super) fn emit(
    output: &mut Vec<u8>,
    (loop_, unsupported_distance, wipe_support): (
        &crate::project_slice::perimeters::classic::chained_loops::ExtrusionLoop,
        Option<f32>,
        Option<&wipe_inward::WipeInwardSupport>,
    ),
    geometry: LayerGeometry<'_>,
    state: &mut EmitState,
    region_perimeters: &[Vec<(i64, i64)>],
) -> Result<(), crate::SliceError> {
    let paths = &loop_.paths;
    let loop_role = loop_.role;
    let small_speed = small_perimeter::speed(paths, &state.options, geometry.scale.factor());
    // `GCode.cpp:5896-5900`: overhang/bridge roles keep the default speed.
    let speed_for_path = |path: &ExtrusionPath| {
        matches!(
            path.role,
            ExtrusionRole::ExternalPerimeter | ExtrusionRole::Perimeter
        )
        .then_some(small_speed)
        .flatten()
    };
    if !state.spiral_vase
        && let Some(scarf) = super::scarf::build(
            paths,
            (loop_role, unsupported_distance),
            geometry,
            &state.options,
            state.layer_index,
        )
    {
        append_wipe_before_external(output, loop_, geometry, state, region_perimeters)?;
        begin_adaptive_loop(state, paths, 0.0, geometry);
        for path in &scarf.paths {
            let _ = super::materialized::emit_subpath(
                output,
                state,
                super::materialized::Emission {
                    path: &path.path,
                    end_clip: 0.0,
                    slope: path.slope,
                    geometry,
                },
                speed_for_path(&path.path),
            )?;
        }
        state.adaptive_pa_flow = None;
        state.adaptive_pa_emitted = false;
        state.wipe_path = wipe_points(&scarf.wipe_paths, geometry);
        wipe_inward::finish_loop(
            output,
            (
                loop_,
                &scarf.wipe_paths,
                (loop_role, scarf.wipe_paths.len(), 0.0),
                wipe_support,
            ),
            geometry,
            state,
        );
        return Ok(());
    }
    // `GCode.cpp:4596-4612,5796-5804`: active vase layers keep complete
    // loops for the full-layer SpiralVase filter.
    let mut remaining_clip = if state.spiral_vase_layer {
        0.0
    } else {
        state.options.seam_gap
    };
    let mut path_count = paths.len();
    while path_count > 0 {
        let length = path_length(&paths[path_count - 1], geometry);
        if length > remaining_clip {
            break;
        }

        path_count -= 1;
        remaining_clip -= length;
    }
    // GCode.cpp:5804-5805 retains the preceding wipe when clipping consumes the loop.
    if path_count == 0 {
        return Ok(());
    }
    append_wipe_before_external(output, loop_, geometry, state, region_perimeters)?;
    begin_adaptive_loop(
        state,
        paths,
        if state.spiral_vase_layer {
            0.0
        } else {
            state.options.seam_gap
        },
        geometry,
    );
    let mut emitted_loop_path = Vec::new();
    for (index, path) in paths[..path_count].iter().enumerate() {
        let end_clip = if index + 1 == path_count {
            remaining_clip
        } else {
            0.0
        };
        let Some(wipe_path) = super::materialized::emit_subpath(
            output,
            state,
            super::materialized::Emission {
                path,
                end_clip,
                slope: None,
                geometry,
            },
            speed_for_path(path),
        )?
        else {
            continue;
        };
        // `GCode.cpp:5979-5990`: upstream appends each emitted path's
        // polyline to `m_wipe.path`, dropping the join point ONLY when
        // it duplicates the previous last point — adjacent sub-paths
        // whose endpoints differ by a scaled unit keep both vertices.
        let reversed = wipe_path.iter().rev().copied();
        let skip_first = emitted_loop_path
            .last()
            .zip(wipe_path.last())
            .is_some_and(|(tail, next)| *tail == *next);
        emitted_loop_path.extend(reversed.skip(usize::from(skip_first)));
    }
    state.adaptive_pa_flow = None;
    state.adaptive_pa_emitted = false;
    state.wipe_path = emitted_loop_path;
    wipe_inward::finish_loop(
        output,
        (
            loop_,
            paths,
            (loop_role, path_count, remaining_clip),
            wipe_support,
        ),
        geometry,
        state,
    );
    Ok(())
}

fn begin_adaptive_loop(
    state: &mut EmitState,
    paths: &[ExtrusionPath],
    end_clip: f64,
    geometry: LayerGeometry<'_>,
) {
    if !state.options.adaptive_pa.enabled {
        return;
    }
    state.adaptive_pa_flow = adaptive_pa::mean(paths, end_clip, geometry.scale);
    state.adaptive_pa_emitted = false;
}

fn wipe_points(paths: &[ExtrusionPath], geometry: LayerGeometry<'_>) -> Vec<super::arc::Point> {
    let mut points = Vec::new();
    for path in paths {
        for point in &path.polyline.points {
            let point = super::arc::Point {
                x: geometry.scale.unscale(point.x),
                y: geometry.scale.unscale(point.y),
            };
            if points.last() != Some(&point) {
                points.push(point);
            }
        }
    }
    points
}

fn append_wipe_before_external(
    output: &mut Vec<u8>,
    loop_: &crate::project_slice::perimeters::classic::chained_loops::ExtrusionLoop,
    geometry: LayerGeometry<'_>,
    state: &mut EmitState,
    region_perimeters: &[Vec<(i64, i64)>],
) -> Result<(), crate::SliceError> {
    let paths = &loop_.paths;
    let loop_role = loop_.role;
    let Some(first) = paths.first() else {
        return Ok(());
    };
    let last = paths.last().unwrap();
    if !state.options.wipe_before_external_loop
        || first.role != ExtrusionRole::ExternalPerimeter
        || region_perimeters.len() <= 1
        || first.polyline.points.len() < 2
        || last.polyline.points.len() < 2
    {
        return Ok(());
    }
    let current = first.polyline.points[0];
    let previous = first.polyline.points[1];
    // `GCode.cpp:5833-5840`: upstream's `paths` passed to this code are
    // already seam-gap clipped (`loop.clip_end(clip_length, &paths)`,
    // `GCode.cpp:5800-5804`), so `paths.back().points.back()` is the
    // CLIPPED end — Ares stores the loop unclipped (the clip is applied
    // per path at emit time), so reproduce the same point by walking
    // `seam_gap` back from the raw end (`clipped_loop_end`). When the
    // result coincides with the start — an unclipped loop whose last
    // point equals the first — upstream falls back to the
    // second-to-last vertex.
    let mut next = clipped_loop_end(paths, state.options.seam_gap, geometry);
    if next.x == current.x && next.y == current.y && last.polyline.points.len() >= 2 {
        next = last.polyline.points[last.polyline.points.len() - 2];
    }
    let mut a = next;
    let mut b = previous;
    let is_hole = loop_role == ExtrusionLoopRole::Hole;
    let counter_clockwise = loop_is_counter_clockwise(paths);
    if is_hole != counter_clockwise {
        std::mem::swap(&mut a, &mut b);
    }
    let mut angle = ccw_angle(current, a, b) / 3.0;
    if is_hole != counter_clockwise {
        angle = -angle;
    }
    let dx = (next.x - current.x) as f64;
    let dy = (next.y - current.y) as f64;
    let length = dx.hypot(dy);
    if length == 0.0 {
        return Ok(());
    }
    // `GCode.cpp:5859`: `dist = std::min(scaled(nozzle_diam) * 0.5,
    // scaled(paths.front().width) * 0.5)` — `scaled<coord_t>`
    // (`Point.hpp:662-668`) divides in the operand's own float domain
    // (the f32 width by f32(SCALING_FACTOR), the f64 nozzle by
    // f64(SCALING_FACTOR)) and truncates each to coord_t before the
    // min; only then is the result halved.
    let scaled_nozzle = (state.options.nozzle_diameter / geometry.scale.factor()) as i64;
    let scaled_width = (first.width / (geometry.scale.factor() as f32)) as i64;
    let maximum = scaled_nozzle.min(scaled_width) as f64 * 0.5;
    let base_x = (current.x as f64 + dx * (2.0 * maximum / length)) as i64 as f64;
    let base_y = (current.y as f64 + dy * (2.0 * maximum / length)) as i64 as f64;
    let (sine, cosine) = angle.sin_cos();
    let x = current.x as f64 + cosine * (base_x - current.x as f64)
        - sine * (base_y - current.y as f64);
    let y = current.y as f64
        + sine * (base_x - current.x as f64)
        + cosine * (base_y - current.y as f64);
    let point = crate::geometry::Point::new(x.round() as i64, y.round() as i64);
    // `discoveredTouchingLines > 1` (`GCode.cpp:5867-5883`): the hop
    // only de-retracts when the point sits inside the model — at least
    // two region perimeter entities must have a segment within one
    // nozzle diameter of it (the wall itself plus a neighbour).
    let nozzle_radius = state.options.nozzle_diameter / geometry.scale.factor();
    let mut touching = 0;
    for polyline in region_perimeters {
        if polyline
            .windows(2)
            .any(|segment| point_segment_distance(point, segment) <= nozzle_radius)
        {
            touching += 1;
            if touching > 1 {
                break;
            }
        }
    }
    if touching <= 1 {
        return Ok(());
    }
    super::path::emit_force_no_extrusion(
        output,
        [(point.x(), point.y()), (current.x, current.y)].into_iter(),
        super::features::PathProperties {
            contour: None,
            mm3_per_mm: 0.0,
            width: first.width,
            height: first.height,
            feature: "Outer wall",
            is_perimeter: true,
            end_clip: 0.0,
            fitting: &[],
            slope: None,
        },
        geometry,
        state,
    )
}

fn clipped_loop_end(
    paths: &[ExtrusionPath],
    mut distance: f64,
    geometry: LayerGeometry<'_>,
) -> Point3 {
    for path in paths.iter().rev() {
        for segment in path.polyline.points.windows(2).rev() {
            let dx = geometry.scale.unscale(segment[1].x - segment[0].x);
            let dy = geometry.scale.unscale(segment[1].y - segment[0].y);
            let length = if path.z_contoured {
                let dz = geometry.scale.unscale(segment[1].z - segment[0].z);
                (dx * dx + dy * dy + dz * dz).sqrt()
            } else {
                dx.hypot(dy)
            };
            if length > distance {
                let ratio = distance / length;
                return Point3 {
                    x: (segment[1].x as f64 + (segment[0].x - segment[1].x) as f64 * ratio) as i64,
                    y: (segment[1].y as f64 + (segment[0].y - segment[1].y) as f64 * ratio) as i64,
                    z: (segment[1].z as f64 + (segment[0].z - segment[1].z) as f64 * ratio) as i64,
                };
            }
            distance -= length;
        }
    }
    paths[0].polyline.points[0]
}

/// Distance from a scaled point to a scaled segment
/// (`AABBTreeLines::all_lines_in_radius` semantics for one segment).
fn point_segment_distance(point: crate::geometry::Point, segment: &[(i64, i64)]) -> f64 {
    let (ax, ay) = (segment[0].0 as f64, segment[0].1 as f64);
    let (bx, by) = (segment[1].0 as f64, segment[1].1 as f64);
    let (px, py) = (point.x() as f64, point.y() as f64);
    let dx = bx - ax;
    let dy = by - ay;
    let length_squared = dx * dx + dy * dy;
    if length_squared <= 0.0 {
        return (px - ax).hypot(py - ay);
    }
    let t = (((px - ax) * dx + (py - ay) * dy) / length_squared).clamp(0.0, 1.0);
    (px - (ax + t * dx)).hypot(py - (ay + t * dy))
}
