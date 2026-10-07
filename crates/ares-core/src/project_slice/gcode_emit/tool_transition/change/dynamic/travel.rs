//! `GCode.cpp:110-197 get_path_of_change_filament`: the safe three-point
//! travel path to `start_end_points[1]` — defaults to (54,0),(54,0),
//! (54,245) when `start_end_points` or `bed_exclude_area` are absent, else
//! detours around merged object X-intervals.

use crate::project_slice::perimeters::classic::traversal::PreparedPostClassicTraversal;

pub(super) fn travel_points(traversal: &PreparedPostClassicTraversal) -> [(f64, f64); 3] {
    let default = [(54.0, 0.0), (54.0, 0.0), (54.0, 245.0)];
    let full = &traversal.resolved.views.full;
    let points = &full.project.print.start_end_points.0;
    if points.len() != 2 {
        return default;
    }
    let exclude = &full.printer.remaining.bed_exclude_area.0;
    if exclude.len() != 4 {
        return default;
    }
    let start = points[0];
    let end = points[1];
    let cutter_y = exclude[2].y + 2.0;

    // Object X-intervals expanded by 2 mm; `can_travel_from_left` fails when
    // any object's min corner is left of the cutter area and above it.
    let mut can_travel_from_left = true;
    let mut intervals: Vec<(f64, f64)> = Vec::new();
    for object in traversal.project.objects() {
        if object.instances().is_empty() {
            continue;
        }
        let (min, max) = object_bounding_box(object);
        if min[0] < start.x && min[1] < cutter_y {
            can_travel_from_left = false;
        }
        let scope = (min[0] - 2.0, max[0] + 2.0);
        // Merge `scope` into all intersecting intervals at once.
        let mut merged: Option<(f64, f64)> = None;
        let mut kept: Vec<(f64, f64)> = Vec::with_capacity(intervals.len() + 1);
        for interval in intervals.drain(..) {
            if interval.1 >= scope.0 && interval.0 <= scope.1 {
                merged = Some(match merged {
                    Some((lo, hi)) => (lo.min(interval.0), hi.max(interval.1)),
                    None => interval,
                });
            } else {
                kept.push(interval);
            }
        }
        kept.push(merged.unwrap_or(scope));
        intervals = kept;
    }
    intervals.sort_by(|left, right| left.0.total_cmp(&right.0));

    // Available X-intervals complementary to the union of objects.
    let mut available: Vec<(f64, f64)> = Vec::with_capacity(intervals.len() + 1);
    let mut cursor = 0.0;
    for interval in &intervals {
        if interval.0 > cursor {
            available.push((cursor, interval.0));
        }
        cursor = interval.1;
    }
    available.push((cursor, 255.0));

    let mut new_path = 255.0;
    for interval in &available {
        if interval.0 > end.x {
            let distance = interval.0 - end.x;
            new_path = if (end.x - new_path).abs() < distance {
                new_path
            } else {
                interval.0
            };
            break;
        } else if interval.1 >= end.x {
            new_path = end.x;
            break;
        } else if !can_travel_from_left && interval.1 < start.x {
            continue;
        } else {
            new_path = interval.1;
        }
    }

    if new_path < start.x {
        [(start.x, cutter_y), (new_path, cutter_y), (new_path, end.y)]
    } else {
        [(new_path, 0.0), (new_path, 0.0), (new_path, end.y)]
    }
}

/// `Model.cpp:1448-1457 bounding_box_exact`: merged per-instance
/// bounding boxes (`instance_bounding_box`, `:1607-1618`) — instance ×
/// volume transform applied to model-part vertices.
fn object_bounding_box(object: &crate::ProjectObject) -> ([f64; 3], [f64; 3]) {
    let mut min = [f64::INFINITY; 3];
    let mut max = [f64::NEG_INFINITY; 3];
    for instance in object.instances() {
        for volume in object.volumes() {
            if volume.volume_type() != crate::ProjectVolumeType::ModelPart {
                continue;
            }
            let matrix = instance.transform().then(volume.transform());
            for vertex in volume.mesh().vertices() {
                let point = matrix.transform_point(*vertex);
                min[0] = min[0].min(point.x);
                min[1] = min[1].min(point.y);
                min[2] = min[2].min(point.z);
                max[0] = max[0].max(point.x);
                max[1] = max[1].max(point.y);
                max[2] = max[2].max(point.z);
            }
        }
    }
    (min, max)
}
