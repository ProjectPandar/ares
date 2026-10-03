//! Normal Grid/Snug `SupportMaterial.cpp::project_support_to_grid`.

use super::pattern::SupportPattern;
use crate::fill::rectilinear::remove_sticks_from_polygon;
use crate::geometry::support_grid::SupportGridParams;
use crate::geometry::{
    ClipperError, CoordinateScale, ExPolygon, JoinType, Polygon, difference_polygons_paths,
    offset_expolygons_paths,
};

pub(super) fn project_support_to_grid(
    slices: &[ExPolygon],
    grid: &SupportGridParams,
    overhangs: &[Polygon],
    buildplate_covered: Option<Vec<Polygon>>,
    scale: CoordinateScale,
) -> Result<(Vec<Polygon>, Vec<Polygon>), ClipperError> {
    let trimming = match buildplate_covered {
        Some(covered) => covered,
        None => {
            offset_expolygons_paths(slices, (1e-4 / scale.factor()) as f32, JoinType::Miter, 3.0)?
        }
    };
    let projected = difference_polygons_paths(overhangs, &trimming)?
        .into_iter()
        .filter_map(remove_sticks_from_polygon)
        .collect::<Vec<_>>();
    let pattern = SupportPattern::new(&projected, &trimming, grid, scale)?;
    Ok((
        pattern.extract_support(&projected, &trimming, grid.expansion_to_slice, true),
        pattern.extract_support(&projected, &trimming, -3, true),
    ))
}
