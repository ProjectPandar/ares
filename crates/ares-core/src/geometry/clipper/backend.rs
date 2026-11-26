// ClipperUtils.cpp:to_paths64, c2_clip, c2_clip_open.
use super::{ClipOperation, ClipperError, FillRule, PolyTree};
use crate::geometry::{ExPolygon, Point, Polygon, Polyline, clipper2 as c2};

pub(super) fn operation(operation: ClipOperation) -> c2::ClipType {
    match operation {
        ClipOperation::Intersection => c2::ClipType::Intersection,
        ClipOperation::Union => c2::ClipType::Union,
        ClipOperation::Difference => c2::ClipType::Difference,
        ClipOperation::Xor => c2::ClipType::Xor,
    }
}

fn feed_points(
    clipper: &mut c2::Clipper64,
    points: impl IntoIterator<Item = c2::Point64>,
    is_clip: bool,
) {
    if is_clip {
        clipper.add_clip_points(points);
    } else {
        clipper.add_subject_points(points);
    }
}

fn to_point64(point: &Point) -> c2::Point64 {
    c2::Point64::new(point.x(), point.y())
}

pub(super) trait PointPaths {
    fn vertex_count(&self) -> usize;
    fn feed_into(&self, clipper: &mut c2::Clipper64, is_clip: bool);
}

impl PointPaths for () {
    fn vertex_count(&self) -> usize {
        0
    }

    fn feed_into(&self, _clipper: &mut c2::Clipper64, _is_clip: bool) {}
}

impl PointPaths for &[Polygon] {
    fn vertex_count(&self) -> usize {
        self.iter().map(|polygon| polygon.points().len()).sum()
    }

    fn feed_into(&self, clipper: &mut c2::Clipper64, is_clip: bool) {
        for polygon in *self {
            feed_points(clipper, polygon.points().iter().map(to_point64), is_clip);
        }
    }
}

impl PointPaths for &[ExPolygon] {
    fn vertex_count(&self) -> usize {
        self.iter()
            .map(|polygon| {
                polygon.contour().points().len()
                    + polygon
                        .holes()
                        .iter()
                        .map(|hole| hole.points().len())
                        .sum::<usize>()
            })
            .sum()
    }

    fn feed_into(&self, clipper: &mut c2::Clipper64, is_clip: bool) {
        for polygon in *self {
            feed_points(
                clipper,
                polygon.contour().points().iter().map(to_point64),
                is_clip,
            );
            for hole in polygon.holes() {
                feed_points(clipper, hole.points().iter().map(to_point64), is_clip);
            }
        }
    }
}

impl PointPaths for &[&ExPolygon] {
    fn vertex_count(&self) -> usize {
        self.iter()
            .map(|polygon| {
                polygon.contour().points().len()
                    + polygon
                        .holes()
                        .iter()
                        .map(|hole| hole.points().len())
                        .sum::<usize>()
            })
            .sum()
    }

    fn feed_into(&self, clipper: &mut c2::Clipper64, is_clip: bool) {
        for polygon in *self {
            feed_points(
                clipper,
                polygon.contour().points().iter().map(to_point64),
                is_clip,
            );
            for hole in polygon.holes() {
                feed_points(clipper, hole.points().iter().map(to_point64), is_clip);
            }
        }
    }
}

impl<A: PointPaths, B: PointPaths> PointPaths for (A, B) {
    fn vertex_count(&self) -> usize {
        self.0.vertex_count() + self.1.vertex_count()
    }

    fn feed_into(&self, clipper: &mut c2::Clipper64, is_clip: bool) {
        self.0.feed_into(clipper, is_clip);
        self.1.feed_into(clipper, is_clip);
    }
}

pub(super) fn polygon(path: &c2::Path64) -> Polygon {
    Polygon::new(path.iter().map(|p| Point::new(p.x, p.y)).collect())
}

pub(super) fn polygons(paths: &c2::Paths64) -> Vec<Polygon> {
    paths.iter().map(polygon).collect()
}

pub(super) fn configured() -> c2::Clipper64 {
    let mut clipper = c2::Clipper64::new();
    clipper.set_preserve_collinear(false);
    clipper
}

pub(super) fn clip_paths<S: PointPaths, C: PointPaths>(
    subject: S,
    clip: C,
    kind: ClipOperation,
    fill: FillRule,
) -> Result<Vec<Polygon>, ClipperError> {
    let mut clipper = configured();
    clipper.reserve_vertices(subject.vertex_count() + clip.vertex_count());
    subject.feed_into(&mut clipper, false);
    clip.feed_into(&mut clipper, true);
    let mut output = c2::Paths64::new();
    clipper.execute(operation(kind), fill, &mut output, None);
    Ok(polygons(&output))
}

pub(super) fn clip_tree<S: PointPaths, C: PointPaths>(
    subject: S,
    clip: C,
    kind: ClipOperation,
    fill: FillRule,
) -> Result<PolyTree, ClipperError> {
    let mut clipper = configured();
    clipper.reserve_vertices(subject.vertex_count() + clip.vertex_count());
    subject.feed_into(&mut clipper, false);
    clip.feed_into(&mut clipper, true);
    let mut output = c2::PolyTree64::new();
    let mut open = c2::Paths64::new();
    clipper.execute_tree(operation(kind), fill, &mut output, &mut open);
    Ok(PolyTree::new(output))
}

pub(super) fn clip_open<C: PointPaths>(
    subject: &[Polyline],
    clip: C,
    kind: ClipOperation,
) -> Result<Vec<Polyline>, ClipperError> {
    let subject_vertices: usize = subject.iter().map(|p| p.points().len()).sum();
    let mut clipper = configured();
    clipper.reserve_vertices(subject_vertices + clip.vertex_count());
    for polyline in subject {
        clipper.add_open_subject_points(polyline.points().iter().map(to_point64));
    }
    clip.feed_into(&mut clipper, true);
    execute_open(clipper, kind)
}

pub(super) fn clip_closed_as_open(
    subject: &[Polygon],
    clip: &[Polygon],
    kind: ClipOperation,
) -> Result<Vec<Polyline>, ClipperError> {
    let subject_vertices: usize = subject
        .iter()
        .map(|polygon| polygon.points().len() + usize::from(!polygon.points().is_empty()))
        .sum();
    let mut clipper = configured();
    clipper.reserve_vertices(subject_vertices + clip.vertex_count());
    for polygon in subject {
        let points = polygon.points();
        clipper.add_open_subject_points(points.iter().chain(points.first()).map(to_point64));
    }
    clip.feed_into(&mut clipper, true);
    execute_open(clipper, kind)
}

fn execute_open(
    mut clipper: c2::Clipper64,
    kind: ClipOperation,
) -> Result<Vec<Polyline>, ClipperError> {
    let mut closed = c2::Paths64::new();
    let mut open = c2::Paths64::new();
    clipper.execute(
        operation(kind),
        FillRule::NonZero,
        &mut closed,
        Some(&mut open),
    );
    Ok(open
        .iter()
        .map(|p| Polyline::new(polygon(p).into_points()))
        .collect())
}
