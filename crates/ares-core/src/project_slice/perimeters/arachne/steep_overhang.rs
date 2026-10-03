//! Width-group detection (`PerimeterGenerator.cpp:58–99,427–450`).

use crate::{
    FloatOrPercent, SliceError,
    geometry::{
        BoundingBox, CoordinateScale, ExPolygon, JoinType, Point, Polyline,
        clip_clipper_expolygons_with_subject_bbox_mode, difference_open_polylines, offset_paths,
    },
    project_slice::perimeters::classic::materialize::ExtrusionPath,
};

pub(super) fn detect(
    paths: &[ExtrusionPath],
    threshold: FloatOrPercent,
    lower: &[ExPolygon],
    scale: CoordinateScale,
) -> Result<bool, SliceError> {
    let mut paths = paths.iter().collect::<Vec<_>>();
    paths.sort_unstable_by(|a, b| a.width.total_cmp(&b.width));
    for group in paths.chunk_by(|a, b| a.width == b.width) {
        let width = f64::from(group[0].width);
        let threshold = match threshold {
            FloatOrPercent::Float(value) => value,
            FloatOrPercent::Percent(percent) => width * percent.0 / 100.0,
        };
        if threshold < 1e-4 {
            return Ok(true);
        }
        let polylines = group
            .iter()
            .map(|path| {
                Polyline::new(
                    path.polyline
                        .points
                        .iter()
                        .map(|point| Point::new(point.x, point.y))
                        .collect(),
                )
            })
            .collect::<Vec<_>>();
        let first = polylines[0].points()[0];
        let mut bounds = BoundingBox::new(first, first);
        for point in polylines.iter().flat_map(|line| line.points()) {
            bounds.merge_point(*point);
        }
        let lower = clip_clipper_expolygons_with_subject_bbox_mode(lower, bounds, true);
        let limit = offset_paths(
            &lower,
            scale.scaled_delta(threshold - 0.5 * width) as f32,
            JoinType::Miter,
            3.0,
        )
        .map_err(geometry_error)?;
        if !difference_open_polylines(&polylines, &limit)
            .map_err(geometry_error)?
            .is_empty()
        {
            return Ok(true);
        }
    }
    Ok(false)
}

fn geometry_error(_: crate::geometry::ClipperError) -> SliceError {
    SliceError::InvalidInput("Arachne steep-overhang geometry failed".to_owned())
}
