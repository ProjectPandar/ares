//! `TreeSupport3D.cpp::create_nodes_from_area/set_points_on_areas`.

mod compaction;
mod model_contact;

use std::collections::VecDeque;

use super::super::model_volumes::TreeModelVolumes;
use super::{TreeSettings, move_inside::move_inside_if_outside, support_element::SupportElement};
use crate::geometry::ClipperError;

pub(super) fn create_nodes_from_area(
    volumes: &mut TreeModelVolumes<'_>,
    config: &TreeSettings,
    move_bounds: &mut [VecDeque<SupportElement>],
) -> Result<(), ClipperError> {
    let (base, above) = move_bounds.split_at_mut(1);
    if let Some(upper) = above.first_mut() {
        for element in upper {
            element.state.bits.marked = false;
        }
    }
    for element in &mut base[0] {
        element.state.result_on_layer = Some(move_inside_if_outside(
            &element.influence_area,
            element.state.next_position,
        ));
        set_points_on_areas(element, above.first_mut());
    }
    for layer in 1..move_bounds.len() {
        if let Some(upper) = move_bounds.get_mut(layer + 1) {
            for element in upper {
                element.state.bits.marked = false;
            }
        }
        for index in 0..move_bounds[layer].len() {
            position_node(volumes, config, move_bounds, (layer, index))?;
        }
    }
    compaction::remove_deleted_elements(move_bounds);
    Ok(())
}

fn position_node(
    volumes: &mut TreeModelVolumes<'_>,
    config: &TreeSettings,
    move_bounds: &mut [VecDeque<SupportElement>],
    location: (usize, usize),
) -> Result<(), ClipperError> {
    let (layer, index) = location;
    let state = &move_bounds[layer][index].state;
    if state.result_on_layer.is_none() {
        if state.bits.to_buildplate
            || ((state.distance_to_top as usize) < config.min_dtt_to_model
                && !state.bits.supports_roof)
        {
            move_bounds[layer][index].state.bits.deleted = true;
        } else if state.bits.to_model_gracious {
            model_contact::gracious(volumes, config, move_bounds, location)?;
        } else {
            model_contact::simple(&mut move_bounds[layer][index]);
        }
    }
    let (below, above) = move_bounds.split_at_mut(layer + 1);
    let element = &mut below[layer][index];
    if !element.state.bits.deleted
        && !element.state.bits.marked
        && element.state.target_height == layer
    {
        element.state.bits.deleted = true;
    }
    if element.state.bits.deleted {
        for &parent in &element.parents {
            above[0][parent as usize].state.result_on_layer = None;
        }
    } else {
        set_points_on_areas(element, above.first_mut());
    }
    Ok(())
}

fn set_points_on_areas(element: &SupportElement, above: Option<&mut VecDeque<SupportElement>>) {
    let Some(point) = element.state.result_on_layer else {
        return;
    };
    let Some(above) = above else {
        return;
    };
    for &index in &element.parents {
        let parent = &mut above[index as usize];
        if parent.state.result_on_layer.is_none() {
            parent.state.result_on_layer =
                Some(move_inside_if_outside(&parent.influence_area, point));
        }
        parent.state.bits.marked = true;
    }
}
