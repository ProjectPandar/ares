use super::ExpansionZone;
use crate::{
    geometry::{
        ClipperError, CoordinateScale, ExPolygon, JoinType, RegionExpansion, closing_ex,
        difference_ex, merge_expansions_into_expolygons, propagate_waves_from_sources,
    },
    project_slice::region_slices::{RegionSurface, RegionSurfaceKind},
};

#[expect(
    clippy::too_many_arguments,
    reason = "the six fields preserve Orca expand_merge_surfaces call semantics"
)]
pub(in crate::project_slice) fn expand_merge_surfaces(
    surfaces: &mut [RegionSurface],
    surface_type: RegionSurfaceKind,
    expansion_zones: &mut [ExpansionZone],
    closing_radius: f32,
    bridge_angle: f64,
    scale: CoordinateScale,
) -> Result<Vec<RegionSurface>, ClipperError> {
    let src = take_surface_expolygons(surfaces, surface_type);
    if src.is_empty() {
        return Ok(Vec::new());
    }

    let mut processed_expolygons_count = 0_u32;
    let mut expansions = Vec::<RegionExpansion>::new();
    for zone in &mut *expansion_zones {
        let mut zone_expansions =
            propagate_waves_from_sources(&src, &zone.expolygons, &zone.parameters, scale)?;
        zone.expanded_into = !zone_expansions.is_empty();
        for expansion in &mut zone_expansions {
            expansion.boundary_id = expansion
                .boundary_id
                .wrapping_add(processed_expolygons_count);
        }
        processed_expolygons_count =
            processed_expolygons_count.wrapping_add(zone.expolygons.len() as u32);
        expansions.append(&mut zone_expansions);
    }

    let expanded = merge_expansions_into_expolygons(src, expansions, scale)?;
    let expanded = closing_ex(&expanded, closing_radius, JoinType::Miter, 3.0)?;
    for zone in &mut *expansion_zones {
        if zone.expanded_into {
            zone.expolygons = difference_ex(&zone.expolygons, &expanded)?;
        }
    }
    Ok(materialize_surfaces(expanded, surface_type, bridge_angle))
}

fn take_surface_expolygons(
    surfaces: &mut [RegionSurface],
    surface_type: RegionSurfaceKind,
) -> Vec<ExPolygon> {
    let count = surfaces
        .iter()
        .filter(|surface| surface.as_parts().0 == surface_type)
        .count();
    let mut output = Vec::with_capacity(count);
    for surface in surfaces {
        if surface.as_parts().0 == surface_type {
            output.push(surface.take_expolygon());
        }
    }
    output
}

fn materialize_surfaces(
    expanded: Vec<ExPolygon>,
    surface_type: RegionSurfaceKind,
    bridge_angle: f64,
) -> Vec<RegionSurface> {
    let mut output = Vec::with_capacity(expanded.len());
    for expolygon in expanded {
        let mut surface = RegionSurface::new(surface_type, expolygon);
        surface.set_bridge_angle(bridge_angle);
        output.push(surface);
    }
    output
}
