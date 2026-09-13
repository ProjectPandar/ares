//! `TimelapsePosPicker` by-layer port (`GCode/TimelapsePosPicker.cpp`).
//!
//! Corpus scope: `pick_pos_for_curr_layer` for by-layer prints — the
//! safe area is the picture extruder's printable region minus the object
//! projections (offset by `extruder_clearance_radius / 2`) and the camera
//! limit areas (first-quadrant bboxes inflated by `√2 · radius / 2`),
//! opened by 5 mm, and the picked position is the boundary candidate with
//! the minimum penalty `|curr − cand|₁ − ⅓ |cand|₁` (camera at the
//! origin). Raft, rod limits, by-object mode and the wipe tower area are
//! not exercised by the smoke corpus (single object, no raft, by-layer,
//! no tower) and stay unported.

use crate::geometry::{self, ExPolygon, Point, Polygon};
use crate::project_slice::perimeters::classic::traversal::PreparedPostClassicTraversal;

use super::footprint;

const FILTER_THRESHOLD_MM: f64 = 5.0;
const CANDIDATE_SEGMENT_MM: f64 = 5.0;

/// Scaled millimeters (`Slic3r::scale_`, SCALING_FACTOR 1e-6).
fn scale(value: f64) -> i64 {
    (value * 1.0e6) as i64
}

fn unscale_trunc(value: i64) -> i64 {
    (f64::from(value as i32) * 1.0e-6) as i64
}

fn scaled_polygon(points: &[(f64, f64)]) -> Polygon {
    Polygon::new(
        points
            .iter()
            .map(|&(x, y)| Point::new(scale(x), scale(y)))
            .collect(),
    )
}

fn bounding_box_polygon(bounds: (f64, f64, f64, f64)) -> Polygon {
    let (min_x, min_y, max_x, max_y) = bounds;
    Polygon::new(vec![
        Point::new(scale(min_x), scale(min_y)),
        Point::new(scale(max_x), scale(min_y)),
        Point::new(scale(max_x), scale(max_y)),
        Point::new(scale(min_x), scale(max_y)),
    ])
}

/// Camera limit (`get_limit_area_for_camera`): the object bbox inflated by
/// `√2 · radius / 2`, constrained to the first quadrant.
fn camera_limit_polygon(bounds: (f64, f64, f64, f64), clearance_radius: f64) -> Polygon {
    let inflate = std::f64::consts::SQRT_2 * clearance_radius * 0.5;
    let (min_x, min_y, max_x, max_y) = (
        (bounds.0 - inflate).max(0.0),
        (bounds.1 - inflate).max(0.0),
        (bounds.2 + inflate).max(0.0),
        (bounds.3 + inflate).max(0.0),
    );
    Polygon::new(vec![
        Point::new(0, 0),
        Point::new(scale(max_x), scale(min_y)),
        Point::new(scale(max_x), scale(max_y)),
        Point::new(scale(min_x), scale(max_y)),
    ])
}

fn polygon_to_expolygon(polygon: Polygon) -> ExPolygon {
    ExPolygon::new(polygon, Vec::new())
}

fn expolygon_contains(expolygon: &ExPolygon, point: Point) -> bool {
    expolygon.contour().contains(&point)
        && !expolygon.holes().iter().any(|hole| hole.contains(&point))
}

/// `pick_pos_internal` for by-layer mode (no path-collision check): the
/// minimum-penalty boundary candidate.
fn pick_pos_internal(current: Point, safe_areas: &[ExPolygon]) -> Option<(i64, i64)> {
    if safe_areas
        .iter()
        .any(|area| expolygon_contains(area, current))
    {
        return Some((unscale_trunc(current.x()), unscale_trunc(current.y())));
    }

    let segment = scale(CANDIDATE_SEGMENT_MM) as f64;
    let mut best: Option<(f64, Point)> = None;
    let mut consider = |candidate: Point| {
        let penalty = (current.x() - candidate.x()).abs() as f64
            + (current.y() - candidate.y()).abs() as f64
            - (candidate.x().abs() + candidate.y().abs()) as f64 / 3.0;
        if best.is_none_or(|(best_penalty, _)| penalty < best_penalty) {
            best = Some((penalty, candidate));
        }
    };

    for area in safe_areas {
        let mut rings = Vec::with_capacity(1 + area.holes().len());
        rings.push(area.contour().clone());
        rings.extend(area.holes().iter().cloned());
        for ring in rings {
            let points = ring.points();
            for index in 0..points.len() {
                let start = points[index];
                let end = points[(index + 1) % points.len()];
                let delta_x = end.x() - start.x();
                let delta_y = end.y() - start.y();
                let length_l1 = (delta_x.abs() + delta_y.abs()) as f64;
                if length_l1 < segment {
                    consider(start);
                    continue;
                }
                let steps = (length_l1 / segment) as i64;
                let step_x = (delta_x as f64 * segment / length_l1) as i64;
                let step_y = (delta_y as f64 * segment / length_l1) as i64;
                for step in 0..=steps {
                    consider(Point::new(
                        start.x() + step_x * step,
                        start.y() + step_y * step,
                    ));
                }
            }
        }
    }
    best.map(|(_, point)| (unscale_trunc(point.x()), unscale_trunc(point.y())))
}

/// The picture-extruder safe-area position for a by-layer print; `None`
/// leaves the default (0, 0) placeholders.
#[allow(dead_code)]
pub(super) fn position(
    traversal: &PreparedPostClassicTraversal,
    picture_extruder: usize,
) -> Option<(i32, i32)> {
    let model_bounds = footprint::model_bounds(traversal)?;
    let printer = &traversal.resolved.views.full.printer;
    let clearance_radius = f64::from(printer.remaining.extruder_clearance_radius.0);
    if let Ok(path) = std::env::var("ARES_DUMP_TIMELAPSE_POS") {
        use std::io::Write;
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
        {
            let _ = writeln!(
                file,
                "bounds={model_bounds:?} picture={picture_extruder} clearance={clearance_radius}"
            );
        }
    }
    let bed = scaled_polygon(
        &printer
            .remaining
            .printable_area
            .0
            .iter()
            .map(|point| (point.x, point.y))
            .collect::<Vec<_>>(),
    );
    let bed_exclude = scaled_polygon(
        &printer
            .remaining
            .bed_exclude_area
            .0
            .iter()
            .map(|point| (point.x, point.y))
            .collect::<Vec<_>>(),
    );

    // `construct_printable_area_by_printer` minus the wipe tower area (no
    // tower in the corpus).
    let mut printable = geometry::difference_polygons_ex(&[bed], &[bed_exclude]).ok()?;
    if let Some(area) = printer
        .remaining
        .extruder_printable_area
        .0
        .get(picture_extruder)
    {
        let extruder_area = scaled_polygon(
            &area
                .iter()
                .map(|point| (point.x, point.y))
                .collect::<Vec<_>>(),
        );
        printable = geometry::intersection_polygons_ex(&[extruder_area], &printable).ok()?;
    }

    // Unplaceable = union(object projections offset by clearance / 2,
    // camera limit polygons); the projection offset takes the first
    // offset polygon like upstream.
    let radius = scale(clearance_radius * 0.5) as f32;
    let projection = geometry::offset_expolygon(
        &polygon_to_expolygon(bounding_box_polygon(model_bounds)),
        radius,
        geometry::JoinType::Miter,
        3.0,
    )
    .ok()?;
    let camera = camera_limit_polygon(model_bounds, clearance_radius);
    let unplaceable =
        geometry::union_expolygons(&projection, &[polygon_to_expolygon(camera)]).ok()?;

    let safe = geometry::difference_ex(&printable, &unplaceable).ok()?;
    let safe = geometry::opening_ex(
        &safe,
        scale(FILTER_THRESHOLD_MM) as f32,
        geometry::JoinType::Miter,
        3.0,
    )
    .ok()?;
    if safe.is_empty() {
        return None;
    }

    let center = Point::new(
        scale((model_bounds.0 + model_bounds.2) * 0.5),
        scale((model_bounds.1 + model_bounds.3) * 0.5),
    );
    pick_pos_internal(center, &safe).map(|(x, y)| (x as i32, y as i32))
}
