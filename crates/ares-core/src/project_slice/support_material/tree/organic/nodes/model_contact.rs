//! `TreeSupport3D.cpp::set_to_model_contact_simple/to_model_gracious`.

use std::collections::VecDeque;

use super::super::super::model_volumes::TreeModelVolumes;
use super::super::{
    TreeSettings, move_inside::move_inside_if_outside, support_element::SupportElement,
};
use crate::geometry::{ClipperError, intersection_polygons_paths};

pub(super) fn simple(element: &mut SupportElement) {
    element.state.result_on_layer = Some(move_inside_if_outside(
        &element.influence_area,
        element.state.next_position,
    ));
}

pub(super) fn gracious(
    volumes: &mut TreeModelVolumes<'_>,
    config: &TreeSettings,
    move_bounds: &mut [VecDeque<SupportElement>],
    first: (usize, usize),
) -> Result<(), ClipperError> {
    let mut location = first;
    let mut last_successful = None;
    loop {
        let element = &move_bounds[location.0][location.1];
        let placeable =
            volumes.placeable_areas(element.state.collision_radius(config), location.0)?;
        if intersection_polygons_paths(&element.influence_area, placeable)?.is_empty() {
            break;
        }
        last_successful = Some(location);
        if element.parents.len() != 1 {
            break;
        }
        location = (location.0 + 1, element.parents[0] as usize);
    }
    let Some(last) = last_successful else {
        let first = &mut move_bounds[first.0][first.1];
        first.state.bits.to_model_gracious = false;
        simple(first);
        return Ok(());
    };
    location = first;
    while location != last {
        let element = &mut move_bounds[location.0][location.1];
        element.state.bits.deleted = true;
        location = (location.0 + 1, element.parents[0] as usize);
    }
    simple(&mut move_bounds[last.0][last.1]);
    Ok(())
}
