use crate::{
    SliceError,
    geometry::{
        ExPolygon, JoinType, difference_ex_refs, difference_ex_with_safety_offset,
        offset_expolygons_paths, offset_paths_tree,
    },
};

use crate::project_slice::region_slices::{RegionSurface, RegionSurfaceKind};

const MITER_LIMIT: f64 = 3.0;
const GEOMETRY_ERROR: &str =
    "surface-type detection geometry is outside the supported Clipper range";

pub(super) fn safety_difference(
    subject: &[ExPolygon],
    clip: &[ExPolygon],
) -> Result<Vec<ExPolygon>, SliceError> {
    difference_ex_with_safety_offset(subject, clip).map_err(geometry_error)
}

pub(super) fn opening_offset(external_width: i64) -> f32 {
    (external_width as f32) / 10.0_f32
}

pub(super) fn opening(input: &[ExPolygon], offset: f32) -> Result<Vec<ExPolygon>, SliceError> {
    let first = offset_expolygons_paths(input, -offset, JoinType::Miter, MITER_LIMIT)
        .map_err(geometry_error)?;
    Ok(
        offset_paths_tree(&first, offset, JoinType::Miter, MITER_LIMIT)
            .map_err(geometry_error)?
            .into_expolygons(crate::geometry::CoordinateScale::Normal),
    )
}

pub(super) fn expolygon_refs(surfaces: &[RegionSurface]) -> Vec<&ExPolygon> {
    surfaces
        .iter()
        .map(|surface| surface.as_parts().1)
        .collect()
}

pub(super) fn expolygons(surfaces: &[RegionSurface]) -> Vec<ExPolygon> {
    surfaces
        .iter()
        .map(|surface| surface.as_parts().1.clone())
        .collect()
}

pub(super) fn fresh(kind: RegionSurfaceKind, expolygons: Vec<ExPolygon>) -> Vec<RegionSurface> {
    expolygons
        .into_iter()
        .map(|expolygon| RegionSurface::new(kind, expolygon))
        .collect()
}

pub(super) fn subtract_surface_refs(
    subject: &[&ExPolygon],
    clip: &[&ExPolygon],
) -> Result<Vec<ExPolygon>, SliceError> {
    difference_ex_refs(subject, clip).map_err(geometry_error)
}

pub(super) fn internal(
    previous: &[ExPolygon],
    top_bottom: &[&ExPolygon],
) -> Result<Vec<RegionSurface>, SliceError> {
    let previous_refs: Vec<&ExPolygon> = previous.iter().collect();
    difference_ex_refs(&previous_refs, top_bottom)
        .map(|expolygons| fresh(RegionSurfaceKind::Internal, expolygons))
        .map_err(geometry_error)
}

pub(super) fn geometry_error(_: crate::geometry::ClipperError) -> SliceError {
    SliceError::InvalidInput(GEOMETRY_ERROR.to_owned())
}
