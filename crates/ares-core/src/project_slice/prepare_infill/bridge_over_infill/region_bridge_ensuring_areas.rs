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
    geometry::{
        ClipperError, CoordinateScale, ExPolygon, JoinType, Polygon, difference_polygons_paths,
        intersection_polygons_polygons_ex, offset_paths, union_safety_offset_ex,
    },
    project_slice::{perimeters::types::Flow, region_slices::RegionSurface},
};

use super::anchored_polygon::scaled_flow_value;

const SHRINK_CONFIGURATION: (JoinType, f64) = (JoinType::Miter, 3.0);

#[derive(Debug, PartialEq)]
pub(in crate::project_slice) struct RegionBridgeEnsuringAreas {
    pub(in crate::project_slice) near_perimeters: Vec<Polygon>,
    pub(in crate::project_slice) additional_ensuring: Vec<ExPolygon>,
}

pub(in crate::project_slice) fn prepare_region_bridge_ensuring_areas(
    fill_surfaces: &[RegionSurface],
    additional_ensuring_areas: &[Polygon],
    solid_infill_flow: Flow,
    scale: CoordinateScale,
) -> Result<RegionBridgeEnsuringAreas, ClipperError> {
    let operations = RegionBridgeEnsuringOperations {
        union: union_safety_offset_ex,
        shrink: shrink_near_perimeters,
        difference: difference_polygons_paths,
        intersection: intersection_polygons_polygons_ex,
    };
    prepare_region_bridge_ensuring_areas_using(
        fill_surfaces,
        additional_ensuring_areas,
        solid_infill_flow,
        scale,
        operations,
    )
}

fn shrink_near_perimeters(polygons: &[Polygon], delta: f32) -> Result<Vec<Polygon>, ClipperError> {
    let (join_type, miter_limit) = SHRINK_CONFIGURATION;
    offset_paths(polygons, delta, join_type, miter_limit)
}

#[cfg(test)]
fn shrink_configuration_for_test() -> (JoinType, f64) {
    SHRINK_CONFIGURATION
}

struct RegionBridgeEnsuringOperations<Union, Shrink, Difference, Intersection> {
    union: Union,
    shrink: Shrink,
    difference: Difference,
    intersection: Intersection,
}

fn prepare_region_bridge_ensuring_areas_using<Union, Shrink, Difference, Intersection>(
    fill_surfaces: &[RegionSurface],
    additional_ensuring_areas: &[Polygon],
    solid_infill_flow: Flow,
    scale: CoordinateScale,
    operations: RegionBridgeEnsuringOperations<Union, Shrink, Difference, Intersection>,
) -> Result<RegionBridgeEnsuringAreas, ClipperError>
where
    Union: FnMut(&[Polygon]) -> Result<Vec<ExPolygon>, ClipperError>,
    Shrink: FnMut(&[Polygon], f32) -> Result<Vec<Polygon>, ClipperError>,
    Difference: FnMut(&[Polygon], &[Polygon]) -> Result<Vec<Polygon>, ClipperError>,
    Intersection: FnMut(&[Polygon], &[Polygon]) -> Result<Vec<ExPolygon>, ClipperError>,
{
    let RegionBridgeEnsuringOperations {
        mut union,
        mut shrink,
        mut difference,
        mut intersection,
    } = operations;
    let fill_polygons = fill_surfaces
        .iter()
        .flat_map(|surface| {
            let expolygon = surface.as_parts().1;
            std::iter::once(expolygon.contour()).chain(expolygon.holes().iter())
        })
        .cloned()
        .collect::<Vec<_>>();
    let unioned = union(&fill_polygons)?;
    let unioned_polygons = flatten_expolygons(unioned);
    let scaled_spacing = scaled_flow_value(solid_infill_flow.spacing, scale);
    let shrunk = shrink(&unioned_polygons, -(scaled_spacing as f32))?;
    let near_perimeters = difference(&unioned_polygons, &shrunk)?;
    let additional_ensuring = intersection(additional_ensuring_areas, &near_perimeters)?;

    Ok(RegionBridgeEnsuringAreas {
        near_perimeters,
        additional_ensuring,
    })
}

fn flatten_expolygons(expolygons: Vec<ExPolygon>) -> Vec<Polygon> {
    expolygons
        .into_iter()
        .flat_map(|expolygon| {
            let (contour, holes) = expolygon.into_parts();
            std::iter::once(contour).chain(holes)
        })
        .collect()
}
