//! Brim.cpp:581–615: outer contour area and no-inner hole exclusion.
use super::inner;
use crate::{
    SliceError,
    geometry::{ExPolygon, JoinType, difference_ex, offset_expolygons},
};

pub(crate) struct OuterArea {
    pub(crate) area: Vec<ExPolygon>,
    pub(crate) forbidden: Vec<ExPolygon>,
    pub(crate) object_islands: Vec<ExPolygon>,
}

pub(crate) fn area(
    source: &[ExPolygon],
    gap: f32,
    outer_width: f32,
    spacing: f32,
    resolution: f64,
) -> Result<OuterArea, SliceError> {
    let mut area = Vec::new();
    let mut forbidden = Vec::new();
    for polygon in source {
        let contour = ExPolygon::new(polygon.contour().clone(), Vec::new());
        let inner_boundary = offset_expolygons(&[contour], gap, JoinType::Round, resolution)
            .map_err(geometry_error)?;
        let outer_boundary =
            offset_expolygons(&inner_boundary, outer_width, JoinType::Round, resolution)
                .map_err(geometry_error)?;
        area.extend(difference_ex(&outer_boundary, &inner_boundary).map_err(geometry_error)?);
        let holes = inner::reversed_holes(polygon);
        // make_brim passes spacing as no_brim_offset; the inset keeps
        // offset_ex's default Miter3, independently of the Round outer ring.
        let inset =
            offset_expolygons(&holes, -spacing, JoinType::Miter, 3.0).map_err(geometry_error)?;
        forbidden.extend(difference_ex(&holes, &inset).map_err(geometry_error)?);
    }
    let object_islands =
        offset_expolygons(source, gap, JoinType::Round, resolution).map_err(geometry_error)?;
    forbidden.extend(object_islands.iter().cloned());
    Ok(OuterArea {
        area,
        forbidden,
        object_islands,
    })
}

fn geometry_error(_: crate::geometry::ClipperError) -> SliceError {
    SliceError::InvalidInput("brim generation failed".into())
}
