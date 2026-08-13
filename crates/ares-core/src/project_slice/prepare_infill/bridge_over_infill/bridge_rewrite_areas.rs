#[cfg(test)]
mod tests;

use crate::{
    geometry::{
        ClipperError, CoordinateScale, JoinType, Polygon, difference_polygons_paths, offset_paths,
    },
    project_slice::perimeters::types::Flow,
};

use super::{anchored_polygon::scaled_flow_value, types::CandidateSurface};

pub(in crate::project_slice) struct UpperBridgeEnsuringInput<'a> {
    pub(in crate::project_slice) surface: &'a CandidateSurface,
    pub(in crate::project_slice) solid_infill_flow: Flow,
}

#[derive(Debug, PartialEq)]
pub(in crate::project_slice) struct BridgeRewriteAreas {
    pub(in crate::project_slice) cut_from_infill: Vec<Polygon>,
    pub(in crate::project_slice) additional_ensuring_areas: Vec<Polygon>,
}

pub(in crate::project_slice) fn collect_bridge_rewrite_areas(
    current: Option<&[CandidateSurface]>,
    upper: Option<&[UpperBridgeEnsuringInput<'_>]>,
    scale: CoordinateScale,
) -> Result<Option<BridgeRewriteAreas>, ClipperError> {
    collect_bridge_rewrite_areas_using(
        current,
        upper,
        scale,
        |polygons, delta| offset_paths(polygons, delta, JoinType::Miter, 3.0),
        difference_polygons_paths,
    )
}

fn collect_bridge_rewrite_areas_using<Shrink, Difference>(
    current: Option<&[CandidateSurface]>,
    upper: Option<&[UpperBridgeEnsuringInput<'_>]>,
    scale: CoordinateScale,
    mut shrink: Shrink,
    mut difference: Difference,
) -> Result<Option<BridgeRewriteAreas>, ClipperError>
where
    Shrink: FnMut(&[Polygon], f32) -> Result<Vec<Polygon>, ClipperError>,
    Difference: FnMut(&[Polygon], &[Polygon]) -> Result<Vec<Polygon>, ClipperError>,
{
    if current.is_none() && upper.is_none() {
        return Ok(None);
    }

    let cut_from_infill = current
        .into_iter()
        .flatten()
        .flat_map(|candidate| candidate.new_polygons.iter().cloned())
        .collect();
    let mut additional_ensuring_areas = Vec::new();
    for input in upper.into_iter().flatten() {
        let scaled_spacing = scaled_flow_value(input.solid_infill_flow.spacing, scale);
        let shrunk = shrink(&input.surface.new_polygons, -(scaled_spacing as f32))?;
        additional_ensuring_areas.extend(difference(&input.surface.new_polygons, &shrunk)?);
    }

    Ok(Some(BridgeRewriteAreas {
        cut_from_infill,
        additional_ensuring_areas,
    }))
}
