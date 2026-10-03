//! `PrintObject::remove_bridges_from_contacts`, Tree's break_bridge=false path.

use super::super::{
    layer_input::to_polygons,
    remove_bridges::{extend_end, extend_start, is_straight, supported_endpoint},
};
use crate::geometry::{
    BoundingBox, ClipperError, Coord, CoordinateScale, ExPolygon, JoinType, Line, Polygon,
    Polyline, difference_ex_polygons_with_safety_offset, difference_open_polylines,
    difference_polygons_paths, offset_open_paths, offset_paths, union_polygons_paths,
};

pub(in super::super) struct BottomBridge<'a> {
    pub(in super::super) polygon: &'a ExPolygon,
    pub(in super::super) angle: f64,
}

pub(in super::super) struct RegionBridges<'a> {
    pub(in super::super) perimeters: &'a [Polyline],
    pub(in super::super) bridge_width_scaled: Coord,
    pub(in super::super) bridge_spacing_scaled: Coord,
    pub(in super::super) bottom_surfaces: &'a [BottomBridge<'a>],
    pub(in super::super) unsupported_edges: &'a [Polyline],
}

pub(super) struct BridgeContactParameters {
    pub(super) extrusion_width_scaled: f32,
    pub(super) max_bridge_length_scaled: f32,
    pub(super) scale: CoordinateScale,
}

pub(super) fn remove_bridges_from_contacts(
    regions: &[RegionBridges<'_>],
    lower_slices: &[ExPolygon],
    overhangs: &mut Vec<ExPolygon>,
    parameters: BridgeContactParameters,
) -> Result<(), ClipperError> {
    let BridgeContactParameters {
        extrusion_width_scaled,
        max_bridge_length_scaled,
        scale,
    } = parameters;
    let lower_polygons = to_polygons(lower_slices);
    let mut all_bridges = Vec::new();
    for region in regions {
        let grown_lower = offset_paths(
            &lower_polygons,
            0.5 * extrusion_width_scaled,
            JoinType::Square,
            0.0,
        )?;
        let perimeters = difference_open_polylines(region.perimeters, &grown_lower)?;
        let width = region.bridge_width_scaled.max(region.bridge_spacing_scaled) as f32;
        let mut bridges = Vec::new();
        for mut perimeter in perimeters {
            if !is_straight(&perimeter) {
                continue;
            }
            extend_start(&mut perimeter, f64::from(extrusion_width_scaled));
            extend_end(&mut perimeter, f64::from(extrusion_width_scaled));
            let length = perimeter
                .points()
                .windows(2)
                .map(|points| Line::new(points[0], points[1]).length())
                .sum::<f64>();
            if supported_endpoint(perimeter.front().unwrap(), lower_slices)
                && supported_endpoint(perimeter.back().unwrap(), lower_slices)
                && length <= f64::from(max_bridge_length_scaled + 10.0)
            {
                bridges.extend(offset_open_paths(
                    &[Polygon::new(perimeter.into_points())],
                    0.5 * width + 10.0,
                    JoinType::Miter,
                    3.0,
                )?);
            }
        }
        let mut bridges = union_polygons_paths(&bridges)?;
        for surface in region.bottom_surfaces {
            if surface.angle == -1.0 {
                continue;
            }
            let bounds = BoundingBox::from_expolygon(surface.polygon).unwrap();
            let width = (bounds.max().x() - bounds.min().x()) as f32;
            let height = (bounds.max().y() - bounds.min().y()) as f32;
            if width < max_bridge_length_scaled && height < max_bridge_length_scaled {
                bridges.extend(to_polygons(std::slice::from_ref(surface.polygon)));
            }
        }
        let edges = region
            .unsupported_edges
            .iter()
            .map(|edge| Polygon::new(edge.points().to_vec()))
            .collect::<Vec<_>>();
        let unsupported =
            offset_open_paths(&edges, (1.2 / scale.factor()) as f32, JoinType::Square, 0.0)?;
        all_bridges.extend(difference_polygons_paths(&bridges, &unsupported)?);
    }
    *overhangs = difference_ex_polygons_with_safety_offset(overhangs, &all_bridges)?;
    Ok(())
}
