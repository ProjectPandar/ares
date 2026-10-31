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

pub(super) fn to_path(points: &[Point]) -> c2::Path64 {
    points
        .iter()
        .map(|p| c2::Point64::new(p.x(), p.y()))
        .collect()
}

pub(super) fn paths(polygons: &[Polygon]) -> c2::Paths64 {
    polygons.iter().map(|p| to_path(p.points())).collect()
}

pub(super) fn expolygon_paths(polygons: &[ExPolygon]) -> c2::Paths64 {
    polygons
        .iter()
        .flat_map(|p| std::iter::once(p.contour()).chain(p.holes()))
        .map(|p| to_path(p.points()))
        .collect()
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

pub(super) fn clip_paths(
    subject: c2::Paths64,
    clip: c2::Paths64,
    kind: ClipOperation,
    fill: FillRule,
) -> Result<Vec<Polygon>, ClipperError> {
    let mut clipper = configured();
    clipper.add_subject(&subject);
    clipper.add_clip(&clip);
    let mut output = c2::Paths64::new();
    clipper.execute(operation(kind), fill, &mut output, None);
    Ok(polygons(&output))
}

pub(super) fn clip_tree(
    subject: c2::Paths64,
    clip: c2::Paths64,
    kind: ClipOperation,
    fill: FillRule,
) -> Result<PolyTree, ClipperError> {
    let mut clipper = configured();
    clipper.add_subject(&subject);
    clipper.add_clip(&clip);
    let mut output = c2::PolyTree64::new();
    let mut open = c2::Paths64::new();
    clipper.execute_tree(operation(kind), fill, &mut output, &mut open);
    Ok(PolyTree::new(output))
}

pub(super) fn clip_open(
    subject: &[Polyline],
    clip: &[Polygon],
    kind: ClipOperation,
) -> Result<Vec<Polyline>, ClipperError> {
    let paths = subject.iter().map(|p| to_path(p.points())).collect();
    let mut clipper = configured();
    clipper.add_open_subject(&paths);
    clipper.add_clip(&self::paths(clip));
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
