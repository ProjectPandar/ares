//! Grid branch of `bottom_contact_layers_and_layer_support_areas`.

use super::{
    bottom_contacts::{BottomContactLayer, BottomContactsInput, detect_bottom_contacts},
    contact_layer::TopContactLayer,
    contact_placement::ContactPlacement,
    layer_input::LayerInput,
    support_projection::project_support_to_grid,
    trim_object::{ObjectTrimming, SupportLayerView, trim_support_layers_by_object},
};
use crate::geometry::support_grid::SupportGridParams;
use crate::geometry::{
    ClipperError, CoordinateScale, JoinType, Polygon, offset_paths, union_polygons_paths,
};

pub(super) struct BottomProjectionInput<'a> {
    pub(super) placement: &'a ContactPlacement<'a>,
    pub(super) layers: &'a [LayerInput<'a>],
    pub(super) top_surfaces: &'a [Vec<Polygon>],
    pub(super) bottom_interface_height: f64,
    pub(super) support_width_scaled: f32,
    pub(super) grid: SupportGridParams,
    pub(super) scale: CoordinateScale,
}

pub(super) struct BottomProjection {
    pub(super) contacts: Vec<BottomContactLayer>,
    pub(super) support_areas: Vec<Vec<Polygon>>,
}

pub(super) fn bottom_contact_layers_and_layer_support_areas(
    input: &BottomProjectionInput<'_>,
    top_contacts: &mut [TopContactLayer],
    mut buildplate_covered: Option<&mut [Vec<Polygon>]>,
    trimming: &ObjectTrimming<'_>,
) -> Result<BottomProjection, ClipperError> {
    if top_contacts.is_empty() {
        return Ok(BottomProjection {
            contacts: Vec::new(),
            support_areas: Vec::new(),
        });
    }
    let mut support_areas = vec![Vec::new(); input.layers.len()];
    let mut contacts = Vec::new();
    let mut overhangs_projection = Vec::new();
    let mut enforcers_projection = Vec::new();
    let mut contact_index = top_contacts.len() as i32 - 1;
    for layer_index in (0..input.layers.len() - 1).rev() {
        let layer = &input.placement.layers[layer_index];
        while contact_index >= 0
            && top_contacts[contact_index as usize].print_z > layer.print_z - 1e-4
        {
            let top = &mut top_contacts[contact_index as usize];
            let mut polygons = std::mem::take(top.contact_polygons.as_mut().unwrap());
            if let Some(enforcers) = &mut top.enforcer_polygons {
                enforcers_projection.append(enforcers);
            }
            polygons.extend(offset_paths(
                top.overhang_polygons.as_ref().unwrap(),
                (1e-4 / input.scale.factor()) as f32,
                JoinType::Miter,
                3.0,
            )?);
            overhangs_projection.extend(union_polygons_paths(&polygons)?);
            contact_index -= 1;
        }
        if overhangs_projection.is_empty() && enforcers_projection.is_empty() {
            continue;
        }
        let overhangs_raw = union_polygons_paths(&overhangs_projection)?;
        let enforcers_raw = union_polygons_paths(&enforcers_projection)?;
        let bottom_projection = if buildplate_covered.is_some() {
            &enforcers_raw
        } else {
            &overhangs_raw
        };
        if !bottom_projection.is_empty()
            && let Some(contact) = detect_bottom_contacts(
                &BottomContactsInput {
                    placement: input.placement,
                    layer_index,
                    top_surfaces: &input.top_surfaces[layer_index],
                    supports_projected: bottom_projection,
                    top_contacts,
                    contact_index,
                    bottom_interface_height: input.bottom_interface_height,
                    support_width_scaled: input.support_width_scaled,
                    scale: input.scale,
                },
                &mut support_areas,
            )?
        {
            contacts.push(contact);
        }
        let covered = buildplate_covered
            .as_deref_mut()
            .map(|layers| std::mem::take(&mut layers[layer_index]));
        let (mut area, propagated) = project_support_to_grid(
            input.layers[layer_index].slices,
            &input.grid,
            &overhangs_raw,
            covered,
            input.scale,
        )?;
        overhangs_projection = propagated;
        let mut enforced_area = if !enforcers_projection.is_empty() {
            let (enforced_area, propagated) = project_support_to_grid(
                input.layers[layer_index].slices,
                &input.grid,
                &enforcers_raw,
                None,
                input.scale,
            )?;
            enforcers_projection = propagated;
            enforced_area
        } else {
            Vec::new()
        };
        if !enforced_area.is_empty() {
            if area.is_empty() {
                area = enforced_area;
            } else {
                area.append(&mut enforced_area);
                area = union_polygons_paths(&area)?;
            }
        }
        support_areas[layer_index] = area;
    }
    contacts.reverse();
    trim_support_layers_by_object(
        trimming,
        contacts.iter_mut().map(|layer| SupportLayerView {
            print_z: layer.print_z,
            bottom_z: layer.bottom_z,
            height: layer.height,
            polygons: &mut layer.polygons,
        }),
    )?;
    Ok(BottomProjection {
        contacts,
        support_areas,
    })
}
