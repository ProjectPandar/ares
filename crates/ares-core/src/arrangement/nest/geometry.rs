//! libnest2d's geometry over the libslic3r backend (`geometry_traits.hpp`,
//! `geometry_traits_nfp.hpp`, `backends/libslic3r/geometries.hpp`, `utils/boost_alg.hpp` at
//! `345efec64`): boxes, areas, the convex hull, point and shape containment, the convex
//! no-fit polygon and the inner bed NFP.

use crate::geometry::{
    ExPolygon, JoinType, Point, Polygon, difference_ex, fixed_gcc_sort_by, point_in_polygon,
    union_expolygons,
};

/// `_Box` with its `defined` flag: a box from two corners is defined; a shape's envelope
/// (`boost::geometry::envelope`) is not, so merging with it keeps the other box.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct NBox {
    pub(crate) min: Point,
    pub(crate) max: Point,
    pub(crate) defined: bool,
}

impl NBox {
    pub(crate) const fn new(min: Point, max: Point) -> Self {
        Self {
            min,
            max,
            defined: true,
        }
    }

    /// The default `_Box`: both corners at the origin, undefined.
    pub(crate) const fn undefined() -> Self {
        Self {
            min: Point::new(0, 0),
            max: Point::new(0, 0),
            defined: false,
        }
    }

    pub(crate) fn width(self) -> i64 {
        self.max.x() - self.min.x()
    }

    pub(crate) fn height(self) -> i64 {
        self.max.y() - self.min.y()
    }

    /// `_Box::center`: each half divided separately, truncated.
    pub(crate) fn center(self) -> Point {
        Point::new(
            self.min.x() / 2 + self.max.x() / 2,
            self.min.y() / 2 + self.max.y() / 2,
        )
    }

    pub(crate) fn area(self) -> f64 {
        self.width() as f64 * self.height() as f64
    }
}

pub(crate) fn add(a: Point, b: Point) -> Point {
    Point::new(a.x() + b.x(), a.y() + b.y())
}

pub(crate) fn sub(a: Point, b: Point) -> Point {
    Point::new(a.x() - b.x(), a.y() - b.y())
}

/// `boundingBox(bb1, bb2)`: an undefined box yields the other.
pub(crate) fn merge_boxes(a: NBox, b: NBox) -> NBox {
    if !a.defined {
        return b;
    }
    if !b.defined {
        return a;
    }
    NBox::new(
        Point::new(a.min.x().min(b.min.x()), a.min.y().min(b.min.y())),
        Point::new(a.max.x().max(b.max.x()), a.max.y().max(b.max.y())),
    )
}

/// `boost::geometry::envelope` of the points (inverse when empty), undefined.
pub(crate) fn envelope<'a>(points: impl Iterator<Item = &'a Point>) -> NBox {
    let (mut min, mut max) = (
        Point::new(i64::MAX, i64::MAX),
        Point::new(i64::MIN, i64::MIN),
    );
    for point in points {
        min = Point::new(min.x().min(point.x()), min.y().min(point.y()));
        max = Point::new(max.x().max(point.x()), max.y().max(point.y()));
    }
    NBox {
        min,
        max,
        defined: false,
    }
}

pub(crate) fn shape_box(shape: &ExPolygon) -> NBox {
    envelope(shape.contour().points().iter())
}

pub(crate) fn shapes_box(shapes: &[ExPolygon]) -> NBox {
    envelope(
        shapes
            .iter()
            .flat_map(|shape| shape.contour().points().iter()),
    )
}

/// `area(path)` for a counter-clockwise backend: positive for counter-clockwise paths.
pub(crate) fn path_area(points: &[Point]) -> f64 {
    if points.len() < 3 {
        return 0.0;
    }
    let mut area = 0.0;
    let mut j = points.len() - 1;
    for i in 0..points.len() {
        let (xj, yj, xi, yi) = (
            points[j].x() as f64,
            points[j].y() as f64,
            points[i].x() as f64,
            points[i].y() as f64,
        );
        area += (xj + xi) * (yj - yi);
        j = i;
    }
    -(area / 2.0)
}

/// `area(polygon)`: the contour plus its (negative) holes.
pub(crate) fn area(shape: &ExPolygon) -> f64 {
    shape
        .holes()
        .iter()
        .fold(path_area(shape.contour().points()), |sum, hole| {
            sum + path_area(hole.points())
        })
}

/// `pointlike::distance`: in double.
pub(crate) fn distance(a: Point, b: Point) -> f64 {
    let (dx, dy) = (b.x() as f64 - a.x() as f64, b.y() as f64 - a.y() as f64);
    (dx * dx + dy * dy).sqrt()
}

/// `_vsort`: by y, then x.
pub(crate) fn vsort_less(a: Point, b: Point) -> bool {
    if a.y() == b.y() {
        a.x() < b.x()
    } else {
        a.y() < b.y()
    }
}

/// `std::max_element` with `_vsort`: the first top-right vertex.
pub(crate) fn rightmost_up_vertex(points: &[Point]) -> Point {
    points
        .iter()
        .copied()
        .reduce(|best, point| if vsort_less(best, point) { point } else { best })
        .unwrap_or(Point::new(0, 0))
}

/// `std::min_element` with `_vsort`: the first bottom-left vertex.
pub(crate) fn leftmost_down_vertex(points: &[Point]) -> Point {
    points
        .iter()
        .copied()
        .reduce(|best, point| if vsort_less(point, best) { point } else { best })
        .unwrap_or(Point::new(0, 0))
}

/// libnest2d's monotone-chain `convexHull` (`geometry_traits.hpp:1049-1110`) of a
/// counter-clockwise path; three or fewer points give an empty hull.
pub(crate) fn convex_hull(points: &[Point]) -> Polygon {
    if points.len() <= 3 {
        return Polygon::new(Vec::new());
    }
    let mut pts = points.to_vec();
    let closed = pts.first() == pts.last();
    if closed {
        pts.pop();
    }
    fixed_gcc_sort_by(&mut pts, |a, b| {
        if a.x() == b.x() {
            a.y() < b.y()
        } else {
            a.x() < b.x()
        }
    });
    let dir = |p: Point, q: Point, r: Point| {
        (i128::from(q.y()) - i128::from(p.y())) * (i128::from(r.x()) - i128::from(p.x()))
            - (i128::from(q.x()) - i128::from(p.x())) * (i128::from(r.y()) - i128::from(p.y()))
    };
    let (mut upper, mut lower): (Vec<Point>, Vec<Point>) = (Vec::new(), Vec::new());
    for &point in &pts {
        while upper.len() > 1 && dir(upper[upper.len() - 2], upper[upper.len() - 1], point) <= 0 {
            upper.pop();
        }
        while lower.len() > 1 && dir(lower[lower.len() - 2], lower[lower.len() - 1], point) >= 0 {
            lower.pop();
        }
        upper.push(point);
        lower.push(point);
    }
    let mut hull: Vec<Point> = lower[..lower.len() - 1].to_vec();
    hull.extend(upper.iter().rev().take(upper.len() - 1));
    if closed {
        hull.push(upper[0]);
    }
    Polygon::new(hull)
}

/// `convexHull(shapes)`: the hull of every contour vertex.
pub(crate) fn shapes_convex_hull(shapes: &[ExPolygon]) -> ExPolygon {
    let points: Vec<Point> = shapes
        .iter()
        .flat_map(|shape| shape.contour().points().iter().copied())
        .collect();
    ExPolygon::new(convex_hull(&points), Vec::new())
}

/// `boost::geometry::within(point, polygon)`: strictly inside the contour and outside every hole.
pub(crate) fn point_within(point: Point, shape: &ExPolygon) -> bool {
    point_in_polygon(point, shape.contour().points()) == 1
        && shape
            .holes()
            .iter()
            .all(|hole| point_in_polygon(point, hole.points()) == 0)
}

/// `boost::geometry::touches(point, polygon)`: on the contour or a hole.
pub(crate) fn point_touches(point: Point, shape: &ExPolygon) -> bool {
    point_in_polygon(point, shape.contour().points()) == -1
        || shape
            .holes()
            .iter()
            .any(|hole| point_in_polygon(point, hole.points()) == -1)
}

/// `boost::geometry::within(polygon, polygon)` for the convex beds arrange packs into: no
/// vertex of `guest` outside `host`, and one strictly inside.
pub(crate) fn shape_within(guest: &ExPolygon, host: &ExPolygon) -> bool {
    let positions: Vec<i32> = guest
        .contour()
        .points()
        .iter()
        .map(|&point| point_in_polygon(point, host.contour().points()))
        .collect();
    !positions.is_empty()
        && positions.iter().all(|&position| position != 0)
        && positions.contains(&1)
}

/// `nfp::merge(shapes)`: `union_ex`.
pub(crate) fn merge(shapes: &[ExPolygon]) -> Vec<ExPolygon> {
    union_expolygons(shapes, &[]).expect("union of arrangement shapes")
}

/// `nfp::subtract`: `diff_ex`.
pub(crate) fn subtract(subject: &[ExPolygon], clip: &[ExPolygon]) -> Vec<ExPolygon> {
    difference_ex(subject, clip).expect("difference of arrangement shapes")
}

/// `shapelike::offset(ExPolygon, distance)`: `offset_ex(sh, distance, jtSquare)`, keeping the
/// first result.
pub(crate) fn offset(shape: &ExPolygon, distance: i64) -> ExPolygon {
    crate::geometry::offset_expolygon(shape, distance as f32, JoinType::Square, 3.0)
        .expect("offset of an arrangement shape")
        .into_iter()
        .next()
        .unwrap_or_else(|| shape.clone())
}

/// `nfpInnerRectBed` (`geometry_traits_nfp.hpp`): where `other`'s top-right reference vertex may
/// go inside `bed`, closed; none when `other` does not fit.
pub(crate) fn nfp_inner_rect_bed(bed: NBox, other: &ExPolygon) -> Option<ExPolygon> {
    let shape = shape_box(other);
    let top = rightmost_up_vertex(other.contour().points());
    let left = top.x() - shape.min.x();
    let right = top.x() - shape.max.x();
    let bottom = shape.max.y() - shape.min.y();
    if bottom > bed.height() || shape.max.x() - shape.min.x() > bed.width() {
        return None;
    }
    Some(ExPolygon::new(
        Polygon::new(vec![
            Point::new(bed.min.x() + left, bed.max.y()),
            Point::new(bed.max.x() + right, bed.max.y()),
            Point::new(bed.max.x() + right, bed.min.y() + bottom),
            Point::new(bed.min.x() + left, bed.min.y() + bottom),
            Point::new(bed.min.x() + left, bed.max.y()),
        ]),
        Vec::new(),
    ))
}

/// `nfpConvexOnly` (`geometry_traits_nfp.hpp`): the Minkowski-sum outline of `sh` and the
/// reversed `other`, by sorting their edges by angle (exactly, as with `boost::rational`), and its
/// top-right reference vertex.
pub(crate) fn nfp_convex_only(sh: &ExPolygon, other: &ExPolygon) -> (ExPolygon, Point) {
    let mut edges: Vec<(Point, Point)> = Vec::new();
    let mut add_edge = |a: Point, b: Point| {
        if a != b {
            edges.push((a, b));
        }
    };
    let (a, b) = (sh.contour().points(), other.contour().points());
    for pair in a.windows(2) {
        add_edge(pair[0], pair[1]);
    }
    if let (Some(&first), Some(&last)) = (a.first(), a.last()) {
        add_edge(last, first);
    }
    for pair in b.windows(2) {
        add_edge(pair[1], pair[0]);
    }
    if let (Some(&first), Some(&last)) = (b.first(), b.last()) {
        add_edge(first, last);
    }
    fixed_gcc_sort_by(&mut edges, |e1, e2| edge_before(*e1, *e2));
    let Some(&(first, second)) = edges.first() else {
        return (
            ExPolygon::new(Polygon::new(Vec::new()), Vec::new()),
            Point::new(0, 0),
        );
    };
    let mut points = vec![first, second];
    let mut top = rightmost_up_vertex(&points);
    for (index, &(start, end)) in edges.iter().enumerate().skip(1) {
        let point = add(end, sub(points[index], start));
        if vsort_less(top, point) {
            top = point;
        }
        points.push(point);
    }
    (ExPolygon::new(Polygon::new(points), Vec::new()), top)
}

/// The edge-angle comparator of `nfpConvexOnly` for a counter-clockwise backend.
fn edge_before(e1: (Point, Point), e2: (Point, Point)) -> bool {
    let (p1, p2) = (sub(e1.1, e1.0), sub(e2.1, e2.0));
    let quadrant = |p: Point| {
        let (cos, sin) = (p.x(), p.y());
        if cos == 0 {
            if sin > 0 { 1 } else { 3 }
        } else if sin == 0 {
            if cos > 0 { 0 } else { 2 }
        } else {
            [0, 3, 1, 2][(usize::from(cos < 0) << 1) + usize::from(sin < 0)]
        }
    };
    let (q1, q2) = (quadrant(p1), quadrant(p2));
    if q1 != q2 {
        return q1 < q2;
    }
    // pcos = sign * cos² / |p|², compared exactly by cross-multiplying.
    let sign: i128 = if q1 == 1 || q1 == 2 { -1 } else { 1 };
    let magnitude =
        |p: Point| i128::from(p.x()) * i128::from(p.x()) + i128::from(p.y()) * i128::from(p.y());
    let (c1, c2) = (i128::from(p1.x()), i128::from(p2.x()));
    let lhs = sign * c1 * c1 * magnitude(p2);
    let rhs = sign * c2 * c2 * magnitude(p1);
    if q1 < 2 { lhs > rhs } else { lhs < rhs }
}
