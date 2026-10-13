//! `PrintConfig.cpp::get_bed_shape_with_excluded_area` and TreeSupport's shift.

use crate::{
    Point3d, ProjectObject, ProjectSettings, SliceError,
    geometry::{CoordinateScale, Point, Polygon, difference_polygons_paths},
};

pub(super) fn polygon(
    settings: &ProjectSettings,
    object: &ProjectObject,
    group: &crate::project::effective_config::types::ResolvedPrintObjectConfig,
    scale: CoordinateScale,
) -> Result<Polygon, SliceError> {
    let points = |input: &crate::Point2dList| {
        let mut polygon = Polygon::new(
            input
                .0
                .iter()
                .map(|point| {
                    Point::new(
                        scale.scaled_delta(point.x) as i64,
                        scale.scaled_delta(point.y) as i64,
                    )
                })
                .collect(),
        );
        if polygon.area() < 0.0 {
            polygon.reverse();
        }
        polygon
    };
    let mut bed = points(&settings.printer.remaining.printable_area);
    let excluded = points(&settings.printer.remaining.bed_exclude_area);
    let mut difference = difference_polygons_paths(std::slice::from_ref(&bed), &[excluded])
        .map_err(|error| SliceError::InvalidInput(format!("support bed: {error:?}")))?;
    if !difference.is_empty() {
        bed = difference.remove(0);
    }
    let center = crate::project_slice::centering::center(object, group, scale)?;
    let translation = object.instances()[*group
        .instance_indices
        .first()
        .expect("print group has a member")]
    .transform()
    .transform_point(Point3d::new(0.0, 0.0, 0.0));
    let shift = Point::new(
        scale.scaled_delta(translation.x) as i64 + center.x(),
        scale.scaled_delta(translation.y) as i64 + center.y(),
    );
    Ok(Polygon::new(
        bed.into_points()
            .into_iter()
            .map(|point| Point::new(point.x() - shift.x(), point.y() - shift.y()))
            .collect(),
    ))
}
