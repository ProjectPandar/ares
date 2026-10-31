//! `tree_supports_generate_paths` (`SupportCommon.cpp:477-703`): sheath
//! loops that start with an anchor cut from a second, inner loop.

use super::RaftEntity;
use super::RaftFillSpec;
use super::geom::clip_polyline_end;
use super::sheath::{SheathFlow, draw_perimeters};
use crate::geometry::{
    ClipperError, CoordinateScale, ExPolygon, JoinType, Point, Polygon, Polyline, ZTaggedPath,
    offset2_ex, shrink_expolygon_with_contour_idx,
};

/// `EPSILON` (`libslic3r.h:48`).
const EPSILON: f64 = 1e-4;

pub(super) fn tree_supports_generate_paths(
    polygons: &[Polygon],
    spec: RaftFillSpec,
    scale: CoordinateScale,
) -> Result<Vec<RaftEntity>, ClipperError> {
    let flow = SheathFlow::new(&spec, scale);
    let clip_length = flow.spacing * 0.15;
    let anchor_length = flow.spacing * 6.0;
    let mut out = Vec::new();
    for expolygon in flow.regions(polygons, scale)? {
        let mut regions = vec![expolygon];
        // Double wall for large branches (`:593-609`); the collection keeps
        // the outer loop and inner loops together, inner first.
        let mut collection = None;
        if spec.tree_double_wall_area > 0.0 && regions[0].area() > spec.tree_double_wall_area {
            let mut entities = Vec::new();
            let level2 = offset2_ex(
                &regions,
                (-1.5 * flow.scaled_width) as f32,
                (0.5 * flow.scaled_width) as f32,
                JoinType::Miter,
                3.0,
            )?;
            if !level2.is_empty() {
                entities.extend(draw_perimeters(&regions[0], clip_length));
                regions = level2;
            }
            collection = Some(entities);
        }
        for region in &regions {
            let polylines = anchored_loops(region, &flow, clip_length, anchor_length)?;
            match &mut collection {
                Some(entities) => entities.extend(polylines),
                None => out.extend(polylines.into_iter().map(RaftEntity::Oriented)),
            }
        }
        if let Some(mut entities) = collection
            && !entities.is_empty()
        {
            entities.reverse();
            out.push(RaftEntity::NoSort(entities));
        }
    }
    Ok(out)
}

/// Region sheath loops and their anchors (`SupportCommon.cpp:614-693`).
fn anchored_loops(
    region: &ExPolygon,
    flow: &SheathFlow,
    clip_length: f64,
    anchor_length: f64,
) -> Result<Vec<Polyline>, ClipperError> {
    // A second loop one width inside supplies the anchor candidates,
    // all oriented CW (`:614-620`).
    let mut anchors =
        shrink_expolygon_with_contour_idx(region, flow.scaled_width as f32, JoinType::Miter, 1.2)?;
    for path in &mut anchors {
        if signed_area(path) > 0.0 {
            path.reverse();
        }
    }
    let loops = std::iter::once(region.contour()).chain(region.holes());
    let mut polylines = Vec::with_capacity(region.holes().len() + 1);
    for (index, polygon) in loops.enumerate() {
        let mut points = polygon.points().to_vec();
        // Orient every loop CW so the anchor extends its end.
        if index == 0 {
            points.reverse();
        }
        points.push(points[0]);
        let mut polyline = Polyline::new(points);
        clip_polyline_end(&mut polyline, clip_length);
        if !polyline.is_valid() {
            continue;
        }
        let mut points = polyline.into_points();
        let anchor = Anchor {
            paths: &anchors,
            loop_index: index as i64,
            max_distance: flow.scaled_width * 3.0,
            length: anchor_length,
        };
        anchor.extend(&mut points);
        // Start with the anchor.
        points.reverse();
        polylines.push(Polyline::new(points));
    }
    Ok(polylines)
}

/// `ClipperLib_Z::Area`: shoelace over the XY of a closed path.
fn signed_area(path: &ZTaggedPath) -> f64 {
    if path.len() < 3 {
        return 0.0;
    }
    let mut area = 0.0;
    let mut previous = path[path.len() - 1];
    for &point in path {
        area += (previous.0 as f64 + point.0 as f64) * (previous.1 as f64 - point.1 as f64);
        previous = point;
    }
    -area * 0.5
}

/// The seam anchor search and cut (`:629-697`).
struct Anchor<'a> {
    paths: &'a [ZTaggedPath],
    loop_index: i64,
    /// `flow.scaled_width() * 3.`
    max_distance: f64,
    /// `spacing * 6.`
    length: f64,
}

impl Anchor<'_> {
    /// Append the anchor to `points` (whose last point is the seam): the
    /// foot of the seam on the closest candidate segment that was offset
    /// from this loop, then up to `length` along that candidate path.
    fn extend(&self, points: &mut Vec<Point>) {
        let seam = xy(points[points.len() - 1]);
        let Some(closest) = self.closest(seam) else {
            return;
        };
        if closest.distance2 >= self.max_distance * self.max_distance {
            return;
        }
        let path = &self.paths[closest.path];
        points.push(to_point(closest.point));
        let mut remaining = self.length - distance(seam, closest.point);
        let mut i = closest.index;
        let mut j = next_index(i, path);
        let mut pi = vertex(path, i);
        let mut pj = vertex(path, j);
        let mut v = sub(pj, pi);
        let mut l = norm(v);
        if remaining < (1.0 - closest.t) * l {
            // Just trim the current line.
            points.push(to_point(add(closest.point, scale2(v, remaining / l))));
            return;
        }
        // Take the rest of the current line, continue with the other lines.
        points.push(Point::new(path[j].0, path[j].1));
        pi = pj;
        i = j;
        while path[i].2 == self.loop_index && remaining > 0.0 {
            j = next_index(i, path);
            pj = vertex(path, j);
            v = sub(pj, pi);
            l = norm(v);
            if i == closest.index {
                // Back at the first segment.
                break;
            }
            if remaining <= l {
                points.push(to_point(add(pi, scale2(v, remaining / l))));
                break;
            }
            points.push(Point::new(path[j].0, path[j].1));
            remaining -= l;
            i = j;
            pi = pj;
        }
    }

    fn closest(&self, seam: [f64; 2]) -> Option<Closest> {
        let mut best = None;
        let mut d2min = f64::MAX;
        let segments = self
            .paths
            .iter()
            .enumerate()
            .flat_map(|(path_index, path)| (0..path.len()).map(move |i| (path_index, path, i)));
        for (path_index, path, i) in segments {
            if let Some(candidate) = self.segment_candidate(path_index, path, i, seam)
                && candidate.distance2 < d2min
            {
                d2min = candidate.distance2;
                best = Some(candidate);
            }
        }
        best
    }

    /// Seam projection onto one eligible anchor segment
    /// (`SupportCommon.cpp:631-657`), before the strict nearest-point test.
    fn segment_candidate(
        &self,
        path_index: usize,
        path: &ZTaggedPath,
        i: usize,
        seam: [f64; 2],
    ) -> Option<Closest> {
        let j = next_index(i, path);
        let (on_i, on_j) = (path[i].2 == self.loop_index, path[j].2 == self.loop_index);
        if !(on_i || on_j) {
            return None;
        }
        let pi = vertex(path, i);
        let v = sub(vertex(path, j), pi);
        let w = sub(seam, pi);
        let l2 = dot(v, v);
        let t = if l2 == 0.0 { 0.0 } else { dot(v, w) / l2 }.clamp(0.0, 1.0);
        if !((on_i || t > EPSILON) && (on_j || t < 1.0 - EPSILON)) {
            return None;
        }
        let point = add(pi, scale2(v, t));
        let d = sub(point, seam);
        let d2 = dot(d, d);
        Some(Closest {
            path: path_index,
            index: i,
            point,
            t,
            distance2: d2,
        })
    }
}

struct Closest {
    path: usize,
    index: usize,
    point: [f64; 2],
    t: f64,
    distance2: f64,
}

/// `next_idx_modulo`.
fn next_index(index: usize, path: &ZTaggedPath) -> usize {
    if index + 1 == path.len() {
        0
    } else {
        index + 1
    }
}

fn vertex(path: &ZTaggedPath, index: usize) -> [f64; 2] {
    [path[index].0 as f64, path[index].1 as f64]
}

fn xy(point: Point) -> [f64; 2] {
    [point.x() as f64, point.y() as f64]
}

/// `Vec2d::cast<coord_t>()` truncates toward zero.
fn to_point(point: [f64; 2]) -> Point {
    Point::new(point[0] as i64, point[1] as i64)
}

fn add(a: [f64; 2], b: [f64; 2]) -> [f64; 2] {
    [a[0] + b[0], a[1] + b[1]]
}

fn sub(a: [f64; 2], b: [f64; 2]) -> [f64; 2] {
    [a[0] - b[0], a[1] - b[1]]
}

fn scale2(a: [f64; 2], factor: f64) -> [f64; 2] {
    [a[0] * factor, a[1] * factor]
}

fn dot(a: [f64; 2], b: [f64; 2]) -> f64 {
    a[0] * b[0] + a[1] * b[1]
}

fn norm(a: [f64; 2]) -> f64 {
    dot(a, a).sqrt()
}

fn distance(a: [f64; 2], b: [f64; 2]) -> f64 {
    norm(sub(a, b))
}
