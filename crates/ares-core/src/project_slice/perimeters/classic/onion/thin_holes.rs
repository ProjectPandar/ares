use crate::{
    SliceError,
    geometry::{
        BoundingBox, ClipperError, CoordinateScale, ExPolygon, JoinType,
        clip_clipper_polygons_with_subject_bbox, intersection_open_polylines, offset2_ex,
    },
};

// PerimeterGenerator.cpp:1699-1717: classify the original normal first shell,
// not the region left after one-wall-top reduction. Hole flags follow normal
// then smaller-width shell order, matching hierarchy materialization.
pub(super) fn classify(
    normal: &[ExPolygon],
    smaller: &[ExPolygon],
    room: f64,
    scale: CoordinateScale,
) -> Result<Vec<bool>, SliceError> {
    if normal.iter().chain(smaller).all(|p| p.holes().is_empty()) {
        return Ok(Vec::new());
    }
    let epsilon = scale.scaled_delta(1.0e-4);
    let reach = offset2_ex(
        normal,
        -(room as f32),
        room as f32 + epsilon as f32,
        JoinType::Miter,
        3.0,
    )
    .map_err(geometry_error)?
    .into_iter()
    .flat_map(|polygon| {
        let (contour, holes) = polygon.into_parts();
        std::iter::once(contour).chain(holes)
    })
    .collect::<Vec<_>>();
    normal
        .iter()
        .chain(smaller)
        .flat_map(ExPolygon::holes)
        .map(|hole| {
            // BoundingBox::offset builds Point(double,double), which rounds.
            let bounds = BoundingBox::from_polygon(hole)
                .unwrap()
                .inflated(epsilon.round() as i64);
            let clipped = clip_clipper_polygons_with_subject_bbox(&reach, bounds);
            intersection_open_polylines(&[hole.split_at_first_point()], &clipped)
                .map(|intersection| intersection.is_empty())
                .map_err(geometry_error)
        })
        .collect()
}

fn geometry_error(_: ClipperError) -> SliceError {
    SliceError::InvalidInput("classic thin-hole reach clipping failed".to_owned())
}
