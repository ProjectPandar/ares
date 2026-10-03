//! Source bbox-clipped support booleans and tiny-fragment trimming.

use crate::geometry::{
    BoundingBox, ClipperError, Coord, CoordinateScale, Point, Polygon,
    clip_clipper_polygons_with_subject_bbox, difference_polygons_paths_with_safety_offset,
    intersection_polygons_paths_with_safety_offset,
};

pub(super) fn difference(
    subject: &[Polygon],
    clip: &[Polygon],
    scale: CoordinateScale,
) -> Result<Vec<Polygon>, ClipperError> {
    difference_polygons_paths_with_safety_offset(subject, &clipped(subject, clip, scale))
}

pub(super) fn intersection(
    subject: &[Polygon],
    clip: &[Polygon],
    scale: CoordinateScale,
) -> Result<Vec<Polygon>, ClipperError> {
    intersection_polygons_paths_with_safety_offset(subject, &clipped(subject, clip, scale))
}

fn clipped(subject: &[Polygon], clip: &[Polygon], scale: CoordinateScale) -> Vec<Polygon> {
    let bounds = BoundingBox::from_polygons(subject)
        .unwrap_or(BoundingBox::new(Point::new(0, 0), Point::new(0, 0)))
        .inflated(scale.scaled_delta(1e-4) as Coord);
    clip_clipper_polygons_with_subject_bbox(clip, bounds)
}

pub(super) fn remove_small(polygons: &mut Vec<Polygon>, scale: CoordinateScale) {
    let tiny = scale.scaled_delta(0.001).powi(2);
    polygons.retain(|polygon| polygon.area().abs() >= tiny);
}
