//! The tower-approach router `WipeTowerIntegration::generate_path_to_wipe_tower`
//! (`GCode.cpp:881-999`, v2.5.0-beta `345efec64`) and the bed it plans against
//! (`shared_printable_area`, `:1012-1020`): shorter squared-length route around the tower box.
use super::super::{motion, timelapse};
use crate::{
    Point2d, SliceError,
    geometry::{
        CoordinateScale, Point as GridPoint, Polygon, intersection_polygons_paths, point_in_polygon,
    },
};
type Point = [i64; 2];

/// `contains(const Polygons&, const Point&, bool border_result)` (`Polygon.cpp:777-787`).
pub(super) fn contains(polygons: &[Polygon], [x, y]: Point, border_result: bool) -> bool {
    let mut inside = 0;
    for polygon in polygons {
        match point_in_polygon(GridPoint::new(x, y), polygon.points()) {
            -1 => return border_result,
            count => inside += count,
        }
    }
    inside % 2 == 1
}

/// `shared_printable_area` (`GCode.cpp:1012-1020`): `Print::get_extruder_shared_printable_polygon`
/// (`Print.cpp:5021-5034`) moved by the plate origin's object point.
pub(super) fn shared_printable_area(
    state: &motion::EmitState,
    context: timelapse::Context<'_>,
    scale: CoordinateScale,
) -> Result<Vec<Polygon>, SliceError> {
    let settings = &context.traversal.resolved.views.full;
    let printer = &settings.printer.remaining;
    let new_scale = |points: &[Point2d]| {
        Polygon::new(
            points
                .iter()
                .map(|p| {
                    GridPoint::new((p.x / scale.factor()) as i64, (p.y / scale.factor()) as i64)
                })
                .collect(),
        )
    };
    let areas = &printer.extruder_printable_area.0;
    let shared = if settings.project.print.nozzle_diameter.0.len() < 2
        || areas.is_empty()
        || areas.iter().any(|area| area.len() < 3)
    {
        vec![new_scale(&printer.printable_area.0)]
    } else {
        areas[1..]
            .iter()
            .try_fold(vec![new_scale(&areas[0])], |shared, area| {
                intersection_polygons_paths(&shared, &[new_scale(area)])
            })
            .map_err(|_| {
                SliceError::InvalidInput("the shared printable area intersection failed".to_owned())
            })?
    };
    let [dx, dy] = motion::tower::object_point_grid([0.0; 2], state, scale);
    Ok(shared
        .into_iter()
        .map(|polygon| {
            Polygon::new(
                polygon
                    .points()
                    .iter()
                    .map(|p| GridPoint::new(p.x() + dx, p.y() + dy))
                    .collect(),
            )
        })
        .collect())
}

/// `generate_path_to_wipe_tower` (`GCode.cpp:887-999`): the route from `start` to `end` around
/// the `[low, high]` avoid box inflated by the routing clearance (`:881`). Without a route
/// (an inflated corner not strictly inside the bed, `:893-901`) it is `end` alone.
pub(super) fn path_to_wipe_tower(
    start: Point,
    end_local: Point,
    [mut low, mut high]: [Point; 2],
    bed: &[Polygon],
    scale: CoordinateScale,
) -> Vec<Point> {
    let offset = (2.0 / scale.factor()) as i64;
    for a in 0..2 {
        low[a] -= offset;
        high[a] += offset;
    }
    let points = [low, [high[0], low[1]], high, [low[0], high[1]]];
    if !points.iter().all(|&p| contains(bed, p, false)) {
        return vec![end_local];
    }
    let direction = if (end_local[0] - low[0]).abs() < (high[0] - low[0]) / 2 {
        -1.0
    } else {
        1.0
    };
    let mm = |p: Point| [scale.unscale(p[0]), scale.unscale(p[1])];
    let intersect = |a: [f64; 2], v: [f64; 2], b: [f64; 2], c: [f64; 2], ray: bool| {
        let w = [c[0] - b[0], c[1] - b[1]];
        let d = [a[0] - b[0], a[1] - b[1]];
        let cross = |a: [f64; 2], b: [f64; 2]| a[0] * b[1] - a[1] * b[0];
        let denominator = cross(v, w);
        if denominator.abs() < 1e-4 {
            return None;
        }
        let t = cross(w, d) / denominator;
        let u = cross(v, d) / denominator;
        (t >= 0.0 && (ray || t <= 1.0) && (0.0..=1.0).contains(&u)).then(|| {
            [
                ((a[0] + t * v[0]) / scale.factor()) as i64,
                ((a[1] + t * v[1]) / scale.factor()) as i64,
            ]
        })
    };
    let Some((end_edge, hit)) = (0..4).find_map(|i| {
        intersect(
            mm(end_local),
            [direction, 0.0],
            mm(points[i]),
            mm(points[(i + 1) % 4]),
            true,
        )
        .map(|p| (i, p))
    }) else {
        return vec![end_local];
    };
    let a = mm(start);
    let b = mm(hit);
    let v = [b[0] - a[0], b[1] - a[1]];
    let entry = (0..4).filter(|&i| i != end_edge).find_map(|i| {
        intersect(a, v, mm(points[i]), mm(points[(i + 1) % 4]), false).map(|p| (i, p))
    });
    let mut path = if let Some((entry_edge, entry)) = entry {
        let around = |forward: bool| {
            let mut i = if forward {
                (entry_edge + 1) % 4
            } else {
                entry_edge
            };
            let end = if forward {
                end_edge
            } else {
                (end_edge + 1) % 4
            };
            let squared = |a: Point, b: Point| {
                let a = mm(a);
                let b = mm(b);
                (a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2)
            };
            let mut length = squared(entry, points[i]);
            let mut path = vec![entry];
            let step = if forward { 1 } else { 3 };
            while i != end {
                let next = (i + step) % 4;
                length += squared(points[i], points[next]);
                path.push(points[i]);
                i = next;
            }
            path.push(points[end]);
            path.push(hit);
            length += squared(hit, points[end]);
            (path, length)
        };
        let (forward, a) = around(true);
        let (backward, b) = around(false);
        if a < b { forward } else { backward }
    } else {
        vec![hit]
    };
    path.push(end_local);
    path
}
