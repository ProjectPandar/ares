use super::{Bridge, group_bridges::group_id};
use crate::{
    geometry::{
        ClipperError, JoinType, Polygon, RegionExpansionEx, offset_paths, offset_paths_tree,
    },
    project_slice::region_slices::{RegionSurface, RegionSurfaceKind},
};

pub(in crate::project_slice) fn merge_bridges(
    mut bridges: Vec<Bridge>,
    bridge_expansions: &[RegionExpansionEx],
    closing_radius: f32,
) -> Result<Vec<RegionSurface>, ClipperError> {
    let mut expansion_ranges = vec![0..0; bridges.len()];
    let mut begin = 0;
    while begin < bridge_expansions.len() {
        let source = bridge_expansions[begin].src_id;
        let mut end = begin + 1;
        while end < bridge_expansions.len() && bridge_expansions[end].src_id == source {
            end += 1;
        }
        expansion_ranges[source as usize] = begin..end;
        begin = end;
    }

    let roots = (0..bridges.len())
        .map(|bridge_id| group_id(&mut bridges, bridge_id as u32) as usize)
        .collect::<Vec<_>>();
    let root_angles = bridges
        .iter()
        .map(|bridge| bridge.angle)
        .collect::<Vec<_>>();
    let mut grouped_polygons = (0..bridges.len())
        .map(|_| Vec::<Polygon>::new())
        .collect::<Vec<_>>();

    for (bridge_id, bridge) in bridges.into_iter().enumerate() {
        let polygons = &mut grouped_polygons[roots[bridge_id]];
        let (contour, mut holes) = bridge.expolygon.into_parts();
        polygons.push(contour);
        polygons.append(&mut holes);
        for expansion in &bridge_expansions[expansion_ranges[bridge_id].clone()] {
            polygons.push(expansion.expolygon.contour().clone());
            polygons.extend(expansion.expolygon.holes().iter().cloned());
        }
    }

    let mut output = Vec::new();
    for (root, polygons) in grouped_polygons.into_iter().enumerate() {
        if roots[root] != root {
            continue;
        }
        let angle = root_angles[root].expect("bridge angle must be calculated before merging");
        let expanded = offset_paths(&polygons, closing_radius, JoinType::Miter, 3.0)?;
        let closed =
            offset_paths_tree(&expanded, -closing_radius, JoinType::Miter, 3.0)?.into_expolygons();
        for expolygon in closed {
            let mut surface = RegionSurface::new(RegionSurfaceKind::BottomBridge, expolygon);
            surface.set_bridge_angle(angle);
            output.push(surface);
        }
    }
    Ok(output)
}
