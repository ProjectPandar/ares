#![cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "the source-cited dependency remains unwired until the bridge transaction"
    )
)]

#[cfg(test)]
mod tests;

use crate::{
    geometry::{ClipperError, ExPolygon, Polygon, difference_ex, difference_ex_polygons},
    project_slice::region_slices::{RegionSurface, RegionSurfaceKind},
};

pub(in crate::project_slice) fn rebuild_internal_infills(
    fill_surfaces: &[RegionSurface],
    cut_from_infill: &[Polygon],
    additional_ensuring: &[ExPolygon],
) -> Result<Vec<RegionSurface>, ClipperError> {
    rebuild_internal_infills_using(
        fill_surfaces,
        cut_from_infill,
        additional_ensuring,
        difference_ex_polygons,
        difference_ex,
    )
}

fn rebuild_internal_infills_using<First, Second>(
    fill_surfaces: &[RegionSurface],
    cut_from_infill: &[Polygon],
    additional_ensuring: &[ExPolygon],
    mut first: First,
    mut second: Second,
) -> Result<Vec<RegionSurface>, ClipperError>
where
    First: FnMut(&[ExPolygon], &[Polygon]) -> Result<Vec<ExPolygon>, ClipperError>,
    Second: FnMut(&[ExPolygon], &[ExPolygon]) -> Result<Vec<ExPolygon>, ClipperError>,
{
    let internal = fill_surfaces
        .iter()
        .filter(|surface| surface.as_parts().0 == RegionSurfaceKind::Internal)
        .map(|surface| surface.as_parts().1.clone())
        .collect::<Vec<_>>();
    let after_cut = first(&internal, cut_from_infill)?;
    let rebuilt = second(&after_cut, additional_ensuring)?;
    Ok(rebuilt.into_iter().map(RegionSurface::internal).collect())
}
