#[cfg(test)]
mod tests;

use crate::{
    ProcessInfillPattern,
    geometry::{
        ClipperError, Coord, CoordinateScale, ExPolygon, JoinType, Polygon, Polyline,
        intersection_open_polylines, intersection_polygons_paths, offset_paths,
    },
    project_slice::region_slices::{RegionSurface, RegionSurfaceKind},
};

const EPSILON: f64 = 1.0e-4;
const MITER_LIMIT: f64 = 3.0;

pub(in crate::project_slice) struct CurrentLayerBridgeRegion<'a> {
    pub(in crate::project_slice) fill_surfaces: &'a [RegionSurface],
    pub(in crate::project_slice) fill_expolygons: &'a [ExPolygon],
    pub(in crate::project_slice) sparse_infill_pattern: ProcessInfillPattern,
}

pub(in crate::project_slice) struct CurrentLayerBridgeContext {
    pub(in crate::project_slice) deep_infill_area: Vec<Polygon>,
    pub(in crate::project_slice) lightning_area: Vec<Polygon>,
    pub(in crate::project_slice) expansion_area: Vec<Polygon>,
    pub(in crate::project_slice) total_fill_area: Vec<Polygon>,
    pub(in crate::project_slice) total_top_area: Vec<Polygon>,
    pub(in crate::project_slice) anchors: Vec<Polyline>,
    pub(in crate::project_slice) internal_unsupported_area: Vec<Polygon>,
}

pub(in crate::project_slice) fn prepare_current_layer_bridge_context(
    deep_infill_area: &[Polygon],
    regions: &[CurrentLayerBridgeRegion<'_>],
    lower_layer_infill_lines: &[Polyline],
    scaled_spacing: Coord,
    scale: CoordinateScale,
) -> Result<CurrentLayerBridgeContext, ClipperError> {
    debug_assert!(scaled_spacing > 0);
    let spacing = scaled_spacing as f64;
    let deep_infill_area = offset_paths(
        deep_infill_area,
        (spacing * 1.5_f64) as f32,
        JoinType::Miter,
        MITER_LIMIT,
    )?;

    let mut lightning_area = Vec::new();
    let mut expansion_area = Vec::new();
    let mut total_fill_area = Vec::new();
    let mut total_top_area = Vec::new();
    for region in regions {
        for surface in region.fill_surfaces {
            let (kind, expolygon, ..) = surface.as_parts();
            let lightning_internal = kind == RegionSurfaceKind::Internal
                && region.sparse_infill_pattern == ProcessInfillPattern::Lightning;
            match kind {
                RegionSurfaceKind::Top => flatten_into(expolygon, &mut total_top_area),
                RegionSurfaceKind::Internal | RegionSurfaceKind::InternalSolid => {
                    flatten_into(expolygon, &mut expansion_area);
                }
                RegionSurfaceKind::Bottom
                | RegionSurfaceKind::BottomBridge
                | RegionSurfaceKind::InternalBridge
                | RegionSurfaceKind::InternalVoid => {}
            }
            if lightning_internal {
                flatten_into(expolygon, &mut lightning_area);
            }
        }
        for expolygon in region.fill_expolygons {
            flatten_into(expolygon, &mut total_fill_area);
        }
    }

    let scaled_epsilon = (EPSILON / scale.factor()) as f32;
    let total_fill_area = closing(total_fill_area, scaled_epsilon)?;
    let expansion_area = closing(expansion_area, scaled_epsilon)?;
    let expansion_area = intersect_expansion_with_deep(&expansion_area, &deep_infill_area)?;
    let spacing = spacing as f32;
    let anchor_area = offset_paths(&expansion_area, -spacing, JoinType::Miter, MITER_LIMIT)?;
    let anchors = clip_lower_lines(lower_layer_infill_lines, &anchor_area)?;
    let internal_unsupported_area = offset_paths(
        &deep_infill_area,
        -(scaled_spacing as f64 * 4.5_f64) as f32,
        JoinType::Miter,
        MITER_LIMIT,
    )?;

    Ok(CurrentLayerBridgeContext {
        deep_infill_area,
        lightning_area,
        expansion_area,
        total_fill_area,
        total_top_area,
        anchors,
        internal_unsupported_area,
    })
}

fn intersect_expansion_with_deep(
    expansion: &[Polygon],
    deep: &[Polygon],
) -> Result<Vec<Polygon>, ClipperError> {
    intersect_expansion_with_deep_using(expansion, deep, intersection_polygons_paths)
}

fn intersect_expansion_with_deep_using<F>(
    expansion: &[Polygon],
    deep: &[Polygon],
    intersect: F,
) -> Result<Vec<Polygon>, ClipperError>
where
    F: FnOnce(&[Polygon], &[Polygon]) -> Result<Vec<Polygon>, ClipperError>,
{
    intersect(expansion, deep)
}

fn clip_lower_lines(
    lower_lines: &[Polyline],
    anchor_area: &[Polygon],
) -> Result<Vec<Polyline>, ClipperError> {
    clip_lower_lines_using(lower_lines, anchor_area, intersection_open_polylines)
}

fn clip_lower_lines_using<F>(
    lower_lines: &[Polyline],
    anchor_area: &[Polygon],
    intersect: F,
) -> Result<Vec<Polyline>, ClipperError>
where
    F: FnOnce(&[Polyline], &[Polygon]) -> Result<Vec<Polyline>, ClipperError>,
{
    intersect(lower_lines, anchor_area)
}

fn closing(paths: Vec<Polygon>, delta: f32) -> Result<Vec<Polygon>, ClipperError> {
    let paths = offset_paths(&paths, delta, JoinType::Miter, MITER_LIMIT)?;
    offset_paths(&paths, -delta, JoinType::Miter, MITER_LIMIT)
}

fn flatten_into(expolygon: &ExPolygon, output: &mut Vec<Polygon>) {
    output.push(expolygon.contour().clone());
    output.extend(expolygon.holes().iter().cloned());
}
