//! Ring selection and handoff shared between the two overloads.
use crate::{
    ProcessSurfaceFillOrder,
    fill::concentric_sparse::nearest_point_index,
    geometry::{
        ClipperError, ExPolygon, JoinType, Point, Polygon, Polyline, offset_paths,
        open_polyline_inside_expolygon, point_in_polygon,
    },
};

pub(super) trait Ring {
    type Path;
    fn open(&self, start: usize) -> Self::Path;
    fn points(path: &Self::Path) -> &[Point];
    fn trim(path: &mut Self::Path, distance: f64);
    fn append(path: &mut Self::Path, next: Self::Path);
    fn reverse(path: &mut Self::Path);
    fn center_stub(
        path: &mut Self::Path,
        ring: &Polygon,
        distance: f64,
    ) -> Result<(), ClipperError>;
}

impl Ring for Polygon {
    type Path = Polyline;
    fn open(&self, start: usize) -> Polyline {
        let points = self.points();
        Polyline::new(
            points[start..]
                .iter()
                .chain(&points[..=start])
                .copied()
                .collect(),
        )
    }
    fn points(path: &Polyline) -> &[Point] {
        path.points()
    }
    fn trim(path: &mut Polyline, distance: f64) {
        path.clip_end(distance);
    }
    fn append(path: &mut Polyline, next: Polyline) {
        path.append_points(next.into_points());
    }
    fn reverse(path: &mut Polyline) {
        path.reverse();
    }
    fn center_stub(path: &mut Polyline, ring: &Polygon, distance: f64) -> Result<(), ClipperError> {
        let center = ring.centroid();
        let last = path.back().unwrap();
        if !offset_paths(
            std::slice::from_ref(ring),
            (-0.5 * distance) as f32,
            JoinType::Miter,
            3.0,
        )?
        .is_empty()
            && center != last
            && distance_squared(last, center).sqrt() <= 2.0 * distance
            && contains(ring, center)
            && reachable(last, center, distance, None)?
        {
            path.points_mut().push(center);
        }
        Ok(())
    }
}

pub(super) fn generate<R: Ring>(
    rings: &[R],
    outlines: &[Polygon],
    distance: i64,
    order: ProcessSurfaceFillOrder,
    original: Option<&ExPolygon>,
) -> Result<Vec<R::Path>, ClipperError> {
    let distance = distance as f64;
    let mut output = Vec::new();
    let mut current: Option<(usize, R::Path)> = None;
    let finish = |(i, mut path): (usize, R::Path)| -> Result<R::Path, ClipperError> {
        R::center_stub(&mut path, &outlines[i], distance)?;
        Ok(path)
    };
    for (i, (ring, outline)) in rings.iter().zip(outlines).enumerate() {
        if outline.points().is_empty() {
            continue;
        }
        let mut path = if let Some((last_ring, ref previous)) = current {
            let last = *R::points(previous).last().unwrap();
            let start = nearest_point_index(last, outline.points());
            let candidate = ring.open(start);
            if R::points(&candidate).len() < 3 {
                continue;
            }
            let next = R::points(&candidate)[0];
            if contains(&outlines[last_ring], next) && reachable(last, next, distance, original)? {
                candidate
            } else {
                output.push(finish(current.take().unwrap())?);
                ring.open(start_corner(outline))
            }
        } else {
            ring.open(start_corner(outline))
        };
        let points = R::points(&path);
        if points.len() < 3 {
            continue;
        }
        let gap = if i + 1 == rings.len() {
            0.5 * distance
        } else {
            distance
        };
        let clip = clip_length(points, gap);
        R::trim(&mut path, clip);
        if R::points(&path).len() < 2 {
            continue;
        }
        if let Some((ring_index, previous)) = &mut current {
            R::append(previous, path);
            *ring_index = i;
        } else {
            current = Some((i, path));
        }
    }
    if let Some(path) = current {
        output.push(finish(path)?);
    }
    if order != ProcessSurfaceFillOrder::Inward {
        output.iter_mut().for_each(R::reverse);
        output.reverse();
    }
    Ok(output)
}

pub(super) fn contains(polygon: &Polygon, point: Point) -> bool {
    point_in_polygon(point, polygon.points()) != 0
}

pub(super) fn distance_squared(a: Point, b: Point) -> f64 {
    let (x, y) = difference(a, b);
    x * x + y * y
}

fn difference(a: Point, b: Point) -> (f64, f64) {
    ((a.x() - b.x()) as f64, (a.y() - b.y()) as f64)
}

fn reachable(
    a: Point,
    b: Point,
    distance: f64,
    original: Option<&ExPolygon>,
) -> Result<bool, ClipperError> {
    let hop = distance_squared(a, b).sqrt();
    if hop <= 1.5 * distance {
        return Ok(true);
    }
    if hop <= 4.0 * distance
        && let Some(domain) = original
    {
        return open_polyline_inside_expolygon(&Polyline::new(vec![a, b]), domain);
    }
    Ok(false)
}

fn start_corner(ring: &Polygon) -> usize {
    let points = ring.points();
    let n = points.len();
    if n < 3 {
        return 0;
    }
    let turn = if ring.is_counter_clockwise() {
        1.0
    } else {
        -1.0
    };
    let mut right = (0.08716, None);
    let mut wide = (1.0, 0);
    for (i, &p) in points.iter().enumerate() {
        let ((ax, ay), (bx, by)) = (
            difference(p, points[(i + n - 1) % n]),
            difference(points[(i + 1) % n], p),
        );
        let (a, b) = ((ax * ax + ay * ay).sqrt(), (bx * bx + by * by).sqrt());
        if a < 1e-6 || b < 1e-6 || turn * (ax * by - ay * bx) <= 0.0 {
            continue;
        }
        let cos = -(ax * bx + ay * by) / (a * b);
        if cos.abs() < right.0 {
            right = (cos.abs(), Some(i));
        }
        if cos > -0.99985 && cos < wide.0 {
            wide = (cos, i);
        }
    }
    right.1.unwrap_or(wide.1)
}

fn clip_length(points: &[Point], gap: f64) -> f64 {
    use std::f64::consts::PI;
    let last = points[points.len() - 1];
    let ((ax, ay), (bx, by)) = (
        difference(last, points[points.len() - 2]),
        difference(points[1], last),
    );
    if (ax * ax + ay * ay).sqrt() < 1e-6 || (bx * bx + by * by).sqrt() < 1e-6 {
        return gap;
    }
    let alpha = (ax * by - ay * bx).abs().atan2(ax * bx + ay * by);
    if alpha > PI / 4.0 && alpha < 2.0 * PI / 3.0 {
        gap / alpha.sin()
    } else {
        gap
    }
}
