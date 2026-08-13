#[cfg(test)]
mod tests;

use crate::{
    geometry::{
        ClipperError, ExPolygon, Polygon, difference_ex_polygons, union_safety_offset_expolygons,
    },
    project_slice::region_slices::{RegionSurface, RegionSurfaceKind},
};

pub(in crate::project_slice) fn recompose_internal_solids(
    fill_surfaces: &[RegionSurface],
    additional_ensuring: &[ExPolygon],
    cut_from_infill: &[Polygon],
) -> Result<Vec<RegionSurface>, ClipperError> {
    recompose_internal_solids_using(
        fill_surfaces,
        additional_ensuring,
        cut_from_infill,
        difference_ex_polygons,
        union_safety_offset_expolygons,
    )
}

fn recompose_internal_solids_using<Difference, Union>(
    fill_surfaces: &[RegionSurface],
    additional_ensuring: &[ExPolygon],
    cut_from_infill: &[Polygon],
    mut difference: Difference,
    mut union: Union,
) -> Result<Vec<RegionSurface>, ClipperError>
where
    Difference: FnMut(&[ExPolygon], &[Polygon]) -> Result<Vec<ExPolygon>, ClipperError>,
    Union: FnMut(&[ExPolygon]) -> Result<Vec<ExPolygon>, ClipperError>,
{
    let mut solids = fill_surfaces
        .iter()
        .filter(|surface| surface.as_parts().0 == RegionSurfaceKind::InternalSolid)
        .map(|surface| surface.as_parts().1.clone())
        .collect::<Vec<_>>();
    solids.extend_from_slice(additional_ensuring);

    let difference_result = difference(&solids, cut_from_infill)?;
    let unioned = union(&difference_result)?;
    Ok(unioned
        .into_iter()
        .map(|expolygon| RegionSurface::new(RegionSurfaceKind::InternalSolid, expolygon))
        .collect())
}
