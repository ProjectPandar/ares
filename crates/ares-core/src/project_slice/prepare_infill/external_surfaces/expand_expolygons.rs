use super::{ExpansionResult, ExpansionZone};
use crate::geometry::{ClipperError, CoordinateScale, ExPolygon, propagate_waves_ex, wave_seeds};

pub(in crate::project_slice) fn expand_expolygons(
    expolygons: &[ExPolygon],
    expansion_zones: &mut [ExpansionZone],
    scale: CoordinateScale,
) -> Result<ExpansionResult, ClipperError> {
    let mut anchors = Vec::new();
    let mut expansions = Vec::new();
    let mut processed_bridges_count = 0_u32;
    for zone in expansion_zones {
        let mut zone_anchors = wave_seeds(
            expolygons,
            &zone.expolygons,
            zone.parameters.tiny_expansion,
            true,
            scale,
        )?;
        let mut zone_expansions =
            propagate_waves_ex(&zone_anchors, &zone.expolygons, &zone.parameters)?;
        for anchor in &mut zone_anchors {
            anchor.boundary = anchor.boundary.wrapping_add(processed_bridges_count);
        }
        for expansion in &mut zone_expansions {
            expansion.boundary_id = expansion.boundary_id.wrapping_add(processed_bridges_count);
        }
        zone.expanded_into = !zone_expansions.is_empty();
        anchors.append(&mut zone_anchors);
        expansions.append(&mut zone_expansions);
        processed_bridges_count =
            processed_bridges_count.wrapping_add(zone.expolygons.len() as u32);
    }
    Ok(ExpansionResult {
        anchors,
        expansions,
    })
}
