//! Grid-style `SupportMaterial.cpp::top_contact_layers`.
//! The owning generator supplies sliced annotations, source region data and
//! source bridging heights before this contact-generation partition runs.

use super::{
    contact_layer::{TopContactLayer, merge_contact_layers},
    contact_placement::ContactPlacement,
    contact_polygons::{ContactPolygons, fill_grid_contact_layer},
    detect_contacts::{ContactDetection, detect_contacts},
    layer_input::{LayerAnnotations, LayerInput, to_polygons},
    overhang_clusters::remove_small_overhangs,
    overhangs::{OverhangDetection, SharpTails, detect_overhangs},
    sharp_tail_propagation::{PropagationInput, propagate_sharp_tails},
};
use crate::geometry::support_grid::SupportGridParams;
use crate::geometry::{ClipperError, CoordinateScale, ExPolygon};
use crate::{ProcessSupportStyle, ProcessSupportType};

pub(super) struct TopContactsInput<'a> {
    pub(super) placement: ContactPlacement<'a>,
    pub(super) layers: &'a [LayerInput<'a>],
    pub(super) annotations: &'a [LayerAnnotations<'a>],
    pub(super) bridging_heights: &'a [Option<f64>],
    pub(super) grid: SupportGridParams,
    pub(super) scale: CoordinateScale,
}

pub(super) fn top_contact_layers(
    input: &TopContactsInput<'_>,
    extrusion_width_scaled: f64,
    sharp_tails: &mut [SharpTails],
) -> Result<Vec<TopContactLayer>, ClipperError> {
    let object = input.placement.object;
    if object.enable_support.0
        && !matches!(
            object.support_type,
            ProcessSupportType::NormalAuto | ProcessSupportType::NormalManual
        )
    {
        return Ok(Vec::new());
    }
    let has_support = object.enable_support.0 || object.enforce_support_layers.0 != 0;
    let num_layers = if has_support { input.layers.len() } else { 1 };
    let parameters = input.placement.parameters;
    let start = if parameters.base_raft_layers + parameters.interface_raft_layers > 0 {
        0
    } else {
        1
    };
    let mut overhangs = vec![Vec::new(); num_layers];
    let mut cantilevers = vec![Vec::new(); num_layers];
    for layer_index in start..num_layers {
        let detected = detect_overhangs(
            &OverhangDetection {
                layer_index,
                layer: &input.layers[layer_index],
                lower: layer_index.checked_sub(1).map(|index| &input.layers[index]),
                object,
                annotations: &input.annotations[layer_index],
                scale: input.scale,
            },
            &mut sharp_tails[layer_index],
        )?;
        overhangs[layer_index] = detected.polygons;
        cantilevers[layer_index] = detected.cantilevers;
    }
    if matches!(
        object.support_type,
        ProcessSupportType::NormalAuto | ProcessSupportType::TreeAuto
    ) {
        propagate_sharp_tails(
            &PropagationInput {
                layers: &input.layers[..num_layers],
                annotations: &input.annotations[..num_layers],
                start_layer: start,
                extrusion_width_scaled,
                scale: input.scale,
            },
            &mut sharp_tails[..num_layers],
            &mut overhangs,
        )?;
    }
    if object.support_remove_small_overhang.0 {
        remove_small_overhangs(
            &mut overhangs,
            &sharp_tails[..num_layers],
            &cantilevers,
            start,
            extrusion_width_scaled,
        )?;
    }
    fill_grid_top_contacts(input, overhangs)
}

fn fill_grid_top_contacts(
    input: &TopContactsInput<'_>,
    overhangs: Vec<Vec<ExPolygon>>,
) -> Result<Vec<TopContactLayer>, ClipperError> {
    let object = input.placement.object;
    let has_support = object.enable_support.0 || object.enforce_support_layers.0 != 0;
    let has_raft = input.placement.parameters.base_raft_layers
        + input.placement.parameters.interface_raft_layers
        > 0;
    let num_layers = if has_support { input.layers.len() } else { 1 };
    let start = if has_raft { 0 } else { 1 };
    let zero_gap_top = input.placement.parameters.zero_gap_interface_top;
    let mut contacts = Vec::new();
    for (layer_index, overhangs) in overhangs
        .into_iter()
        .enumerate()
        .take(num_layers)
        .skip(start)
    {
        let lower_slices = if layer_index == 0 {
            &[][..]
        } else {
            input.layers[layer_index - 1].slices
        };
        let mut overhangs = to_polygons(&overhangs);
        let detected = detect_contacts(
            &ContactDetection {
                layer_index,
                layer: &input.layers[layer_index],
                lower_slices,
                annotations: &input.annotations[layer_index],
                raft_expansion_scaled: (object.raft_expansion.0 / input.scale.factor()) as f32,
                gap_xy_scaled: (object.support_object_xy_distance.0 / input.scale.factor()) as f32,
            },
            &mut overhangs,
        )?;
        if detected.polygons.is_empty() && overhangs.is_empty() {
            continue;
        }
        let Some((mut layer, mut bridging_layer)) = input
            .placement
            .new_contact_layer(layer_index, input.bridging_heights[layer_index])
        else {
            continue;
        };
        fill_grid_contact_layer(
            &mut layer,
            ContactPolygons {
                scale: input.scale,
                layer_index,
                grid: input.grid,
                reduce_interfaces: super::style::resolve(object.support_type, object.support_style)
                    != ProcessSupportStyle::Snug
                    && layer_index > 0
                    && !zero_gap_top,
                no_interface_offset: detected.no_interface_offset,
                lower_layer_polygons: &to_polygons(lower_slices),
                trimming_polygons: &detected.slices_margin.polygons,
                all_trimming_polygons: &detected.slices_margin.all_polygons,
                contact_polygons: &detected.polygons,
                overhang_polygons: overhangs,
                enforcer_polygons: detected.enforcer_polygons,
            },
        )?;
        if let Some(bridging) = &mut bridging_layer {
            bridging.polygons.clone_from(&layer.polygons);
            bridging
                .contact_polygons
                .clone_from(&layer.contact_polygons);
            bridging
                .overhang_polygons
                .clone_from(&layer.overhang_polygons);
            bridging
                .enforcer_polygons
                .clone_from(&layer.enforcer_polygons);
        }
        // Preserve native per-object slot enumeration before the Z sort.
        contacts.push(layer);
        contacts.extend(bridging_layer);
    }
    merge_contact_layers(
        contacts,
        input.placement.parameters.first_print_layer_height,
        input.placement.support_layer_height_min,
    )
}
