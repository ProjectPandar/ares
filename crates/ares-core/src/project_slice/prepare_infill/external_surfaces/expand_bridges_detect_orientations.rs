use super::{
    ExpansionZone, detect_bridge_directions::detect_bridge_directions,
    expand_expolygons::expand_expolygons, group_bridges::get_grouped_bridges,
    merge_bridges::merge_bridges,
};
use crate::{
    geometry::{ClipperError, CoordinateScale, difference_ex},
    project_slice::region_slices::{RegionSurface, RegionSurfaceKind},
};

pub(in crate::project_slice) fn expand_bridges_detect_orientations(
    surfaces: &mut [RegionSurface],
    expansion_zones: &mut [ExpansionZone],
    closing_radius: f32,
    scale: CoordinateScale,
) -> Result<Vec<RegionSurface>, ClipperError> {
    let bridge_expolygons = surfaces
        .iter_mut()
        .filter(|surface| surface.as_parts().0 == RegionSurfaceKind::BottomBridge)
        .map(RegionSurface::take_expolygon)
        .collect::<Vec<_>>();
    if bridge_expolygons.is_empty() {
        return Ok(Vec::new());
    }

    let mut expansion_result = expand_expolygons(&bridge_expolygons, expansion_zones, scale)?;
    let mut bridges = get_grouped_bridges(bridge_expolygons, &expansion_result.expansions)?;
    expansion_result
        .anchors
        .sort_by_key(|anchor| (anchor.src, anchor.boundary));
    detect_bridge_directions(
        &expansion_result.anchors,
        &mut bridges,
        expansion_zones,
        scale,
    )?;
    expansion_result
        .expansions
        .sort_by_key(|expansion| (expansion.src_id, expansion.boundary_id));
    let output = merge_bridges(bridges, &expansion_result.expansions, closing_radius)?;
    let output_expolygons = output
        .iter()
        .map(|surface| surface.as_parts().1.clone())
        .collect::<Vec<_>>();
    for zone in expansion_zones {
        if zone.expanded_into {
            zone.expolygons = difference_ex(&zone.expolygons, &output_expolygons)?;
        }
    }
    Ok(output)
}
