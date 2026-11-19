//! Brim.cpp:592–607: explicit OuterAndInner, contour Round and hole Miter3.
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
    (outer_width, hole_width): (f32, f32),
    resolution: f64,
) -> Result<CombinedArea, SliceError> {
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
        area.extend(inner::hole_area(&holes, gap, hole_width)?);
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
