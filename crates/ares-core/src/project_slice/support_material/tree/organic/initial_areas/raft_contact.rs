//! `TreeSupport3D.cpp::generate_raft_contact/finalize_raft_contact`.

use std::collections::VecDeque;

use super::generation::ObjectInputs;
use crate::{
    geometry::{BoundingBox, ClipperError, Coord, EdgeGrid, JoinType, offset_expolygons_paths},
    project_slice::support_material::{
        layer_input::to_polygons,
        tree::organic::{interface_placer::InterfacePlacer, support_element::SupportElement},
    },
};

const EPSILON: f64 = 1e-4;

pub(in super::super) fn generate(
    input: &ObjectInputs<'_>,
    interface: &mut InterfacePlacer<'_>,
) -> Result<Option<usize>, ClipperError> {
    if input.options.raft_layers.0 <= 0 || input.slices.is_empty() {
        return Ok(None);
    }
    let config = interface.config;
    let mut layer = config.raft_layers.len() - 1;
    while layer > 0 && config.raft_layers[layer] > interface.slicing.raft_contact_top_z + EPSILON {
        layer -= 1;
    }
    let expansion = input.options.raft_expansion.0;
    let polygons = if expansion > 0.0 {
        offset_expolygons_paths(
            &input.slices[0],
            input.scale.scaled_delta(expansion) as f32,
            JoinType::Miter,
            3.0,
        )?
    } else {
        to_polygons(&input.slices[0])
    };
    interface.add_roof(polygons, layer, 0);
    Ok(Some(layer))
}

pub(super) fn finalize(
    input: &ObjectInputs<'_>,
    contact_layer: Option<usize>,
    interface: &mut InterfacePlacer<'_>,
    move_bounds: &mut [VecDeque<SupportElement>],
) -> Result<(), ClipperError> {
    let Some(contact_layer) = contact_layer else {
        return Ok(());
    };
    let first_tree_layer =
        interface.slicing.base_raft_layers + interface.slicing.interface_raft_layers - 1;
    for (contact, tips) in interface
        .roofs
        .top_contacts
        .iter_mut()
        .zip(&mut *move_bounds)
        .take(first_tree_layer)
    {
        *contact = None;
        tips.clear();
    }
    let expansion = input.options.raft_expansion.0;
    if expansion > 0.0 {
        let index =
            interface.roofs.top_contacts[contact_layer].expect("generated raft contact exists");
        let polygons = &interface.roofs.storage[index].polygons;
        let bounds = BoundingBox::from_polygons(polygons)
            .expect("raft contact has first-layer slices")
            .inflated(input.scale.scaled_delta(EPSILON) as Coord);
        let grid = EdgeGrid::new_from_contours(
            polygons.iter().map(|polygon| polygon.points()),
            bounds.min(),
            bounds.max(),
            input.scale.scaled_delta(10.0) as Coord,
        )?;
        let threshold = input.scale.scaled_delta(expansion) * 2.0;
        let tips = &mut move_bounds[first_tree_layer];
        for _ in 0..tips.len() {
            let tip = tips.pop_front().expect("each original tip is visited once");
            let point = tip
                .state
                .result_on_layer
                .expect("initial tip result is set");
            // Both native closed-edge queries share the distance/sign search.
            let inside = grid
                .closest_point_signed_distance(point, threshold as Coord)?
                .is_some_and(|result| result.distance < -0.0);
            if !inside {
                tips.push_back(tip);
            }
        }
    }
    Ok(())
}
