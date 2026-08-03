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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::project_slice) enum GeometryStep {
    TopSafetyDifference,
    TopShrink,
    TopExpand,
    BottomSafetyDifference,
    BottomShrink,
    BottomExpand,
    CrackIntersection,
    SingletonCrackErosion,
    ContainmentDifference,
    ResidualDifference,
    CollectionResidualErosion,
    SingletonCrackExpansion,
    BottomSubtraction,
    TopDifference,
    InternalDifference,
    FillTopIntersection,
    FillBottomIntersection,
    FillBottomBridgeIntersection,
    FillInternalIntersection,
}

pub(super) fn safety_difference(
    subject: &[ExPolygon],
    clip: &[ExPolygon],
    step: GeometryStep,
) -> Result<Vec<ExPolygon>, SliceError> {
    observe(step)?;
    difference_ex_with_safety_offset(subject, clip).map_err(geometry_error)
}

pub(super) fn opening_offset(external_width: i64) -> f32 {
    (external_width as f32) / 10.0_f32
}

pub(super) fn opening(
    input: &[ExPolygon],
    offset: f32,
    shrink: GeometryStep,
    expand: GeometryStep,
) -> Result<Vec<ExPolygon>, SliceError> {
    observe(shrink)?;
    let first = offset_expolygons_paths(input, -offset, JoinType::Miter, MITER_LIMIT)
        .map_err(geometry_error)?;
    observe(expand)?;
    Ok(
        offset_paths_tree(&first, offset, JoinType::Miter, MITER_LIMIT)
            .map_err(geometry_error)?
            .into_expolygons(),
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
    observe(GeometryStep::TopDifference)?;
    difference_polygons_ex(subject, clip).map_err(geometry_error)
}

pub(super) fn internal(
    previous: &[ExPolygon],
    top_bottom: &[Polygon],
) -> Result<Vec<RegionSurface>, SliceError> {
    observe(GeometryStep::InternalDifference)?;
    difference_ex_polygons(previous, top_bottom)
        .map(|expolygons| fresh(RegionSurfaceKind::Internal, expolygons))
        .map_err(geometry_error)
}

pub(super) fn observe(step: GeometryStep) -> Result<(), SliceError> {
    #[cfg(test)]
    if super::tests::observe_step(step) {
        return Err(geometry_error_value());
    }
    #[cfg(not(test))]
    let _ = step;
    Ok(())
}

pub(super) fn geometry_error(_: crate::geometry::ClipperError) -> SliceError {
    geometry_error_value()
}

fn geometry_error_value() -> SliceError {
    SliceError::InvalidInput(GEOMETRY_ERROR.to_owned())
}
