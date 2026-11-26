use crate::{
    SliceError,
    geometry::{
        ExPolygon, JoinType, Polygon, difference_ex_polygons, difference_ex_with_safety_offset,
        difference_polygons_ex, offset_expolygons_paths, offset_paths_tree,
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

pub(super) fn paths(surfaces: &[RegionSurface]) -> Vec<Polygon> {
    let mut paths = Vec::new();
    for surface in surfaces {
        let (_, expolygon, ..) = surface.as_parts();
        paths.push(expolygon.contour().clone());
        paths.extend(expolygon.holes().iter().cloned());
    }
    paths
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

pub(super) fn subtract_paths(
    subject: &[Polygon],
    clip: &[Polygon],
) -> Result<Vec<ExPolygon>, SliceError> {
    difference_polygons_ex(subject, clip).map_err(geometry_error)
}

pub(super) fn internal(
    previous: &[ExPolygon],
    top_bottom: &[Polygon],
) -> Result<Vec<RegionSurface>, SliceError> {
    difference_ex_polygons(previous, top_bottom)
        .map(|expolygons| fresh(RegionSurfaceKind::Internal, expolygons))
        .map_err(geometry_error)
}

pub(super) fn geometry_error(_: crate::geometry::ClipperError) -> SliceError {
    SliceError::InvalidInput(GEOMETRY_ERROR.to_owned())
}
