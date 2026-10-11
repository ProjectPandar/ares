//! Brim.cpp:448-570: explicit OuterAndInner, contour Round and hole Miter3.
use super::inner;
use crate::{
    SliceError,
    geometry::{ExPolygon, JoinType, difference_ex, offset_expolygons},
};

pub(crate) struct CombinedArea {
    pub(crate) area: Vec<ExPolygon>,
    pub(crate) object_islands: Vec<ExPolygon>,
}

pub(crate) fn area(
    source: &[ExPolygon],
    gap: f32,
    width: f32,
    spacing: f32,
    resolution: f64,
) -> Result<CombinedArea, SliceError> {
    // Explicit combined mode has no auto additions/clamp; line527 still
    // re-quantizes the outer width. Hole offsets retain the original width.
    let outer_width = (width / spacing / 2.0).floor() * spacing * 2.0;
    let mut area = Vec::new();
    for polygon in source {
        let contour = ExPolygon::new(polygon.contour().clone(), Vec::new());
        let inner_boundary = offset_expolygons(&[contour], gap, JoinType::Round, resolution)
            .map_err(geometry_error)?;
        let outer_boundary =
            offset_expolygons(&inner_boundary, outer_width, JoinType::Round, resolution)
                .map_err(geometry_error)?;
        area.extend(difference_ex(&outer_boundary, &inner_boundary).map_err(geometry_error)?);
        let holes = inner::reversed_holes(polygon);
        area.extend(inner::hole_area(&holes, gap, width)?);
    }
    let object_islands =
        offset_expolygons(source, gap, JoinType::Round, resolution).map_err(geometry_error)?;
    Ok(CombinedArea {
        area,
        object_islands,
    })
}

fn geometry_error(_: crate::geometry::ClipperError) -> SliceError {
    SliceError::InvalidInput("brim generation failed".into())
}
