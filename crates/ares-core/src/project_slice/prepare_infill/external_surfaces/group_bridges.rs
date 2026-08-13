use super::Bridge;
use crate::geometry::{
    BoundingBox, ClipperError, ExPolygon, RegionExpansionEx, intersection_polygons_paths,
};

pub(in crate::project_slice) fn group_id(bridges: &mut [Bridge], mut src_id: u32) -> u32 {
    let mut group_id = bridges[src_id as usize].group_id;
    while group_id != src_id {
        src_id = group_id;
        group_id = bridges[src_id as usize].group_id;
    }
    bridges[src_id as usize].group_id = group_id;
    group_id
}

fn group_bridge_pair(
    bridges: &mut [Bridge],
    current: &RegionExpansionEx,
    candidate: &RegionExpansionEx,
    current_bounding_box: BoundingBox,
    candidate_bounding_box: BoundingBox,
) -> Result<(), ClipperError> {
    if current.src_id != candidate.src_id
        && !(current_bounding_box.max().x() < candidate_bounding_box.min().x()
            || current_bounding_box.min().x() > candidate_bounding_box.max().x()
            || current_bounding_box.max().y() < candidate_bounding_box.min().y()
            || current_bounding_box.min().y() > candidate_bounding_box.max().y())
        && !intersection_polygons_paths(
            std::slice::from_ref(current.expolygon.contour()),
            std::slice::from_ref(candidate.expolygon.contour()),
        )?
        .is_empty()
    {
        let id = group_id(bridges, current.src_id);
        let id2 = group_id(bridges, candidate.src_id);
        if id < id2 {
            bridges[id2 as usize].group_id = id;
        } else {
            bridges[id as usize].group_id = id2;
        }
    }
    Ok(())
}

pub(in crate::project_slice) fn get_grouped_bridges(
    bridge_expolygons: Vec<ExPolygon>,
    bridge_expansions: &[RegionExpansionEx],
) -> Result<Vec<Bridge>, ClipperError> {
    let mut result = Vec::with_capacity(bridge_expansions.len());
    for (group_id, expolygon) in bridge_expolygons.into_iter().enumerate() {
        result.push(Bridge {
            expolygon,
            group_id: group_id as u32,
            angle: None,
        });
    }

    let mut expansion_index = 0;
    while expansion_index != bridge_expansions.len() {
        let boundary_region_begin = expansion_index;
        let boundary_id = bridge_expansions[expansion_index].boundary_id;
        let boundary_region_end = bridge_expansions[expansion_index + 1..]
            .iter()
            .position(|expansion| expansion.boundary_id != boundary_id)
            .map_or(bridge_expansions.len(), |offset| {
                expansion_index + offset + 1
            });
        let bounding_boxes = bridge_expansions[boundary_region_begin..boundary_region_end]
            .iter()
            .map(|expansion| {
                BoundingBox::from_polygon(expansion.expolygon.contour())
                    .expect("bridge expansion contour must be nonempty")
            })
            .collect::<Vec<_>>();

        while expansion_index != boundary_region_end {
            for candidate_index in expansion_index + 1..boundary_region_end {
                let current = &bridge_expansions[expansion_index];
                let candidate = &bridge_expansions[candidate_index];
                let current_bounding_box = bounding_boxes[expansion_index - boundary_region_begin];
                let candidate_bounding_box =
                    bounding_boxes[candidate_index - boundary_region_begin];
                group_bridge_pair(
                    &mut result,
                    current,
                    candidate,
                    current_bounding_box,
                    candidate_bounding_box,
                )?;
            }
            expansion_index += 1;
        }
    }
    Ok(result)
}
