//! `SupportMaterial.cpp::detect_bottom_contacts` and bottom-gap synchronization.

use super::{contact_layer::TopContactLayer, contact_placement::ContactPlacement};
use crate::geometry::{
    ClipperError, CoordinateScale, JoinType, Polygon, difference_polygons_paths,
    intersection_polygons_paths, offset_paths,
};
use crate::project_slice::layers::PlannedLayer;

const EPSILON: f64 = 1e-4;

pub(super) struct BottomContactLayer {
    pub(super) print_z: f64,
    pub(super) bottom_z: f64,
    pub(super) height: f64,
    pub(super) bridging: bool,
    pub(super) polygons: Vec<Polygon>,
}

pub(super) struct BottomContactsInput<'a> {
    pub(super) placement: &'a ContactPlacement<'a>,
    pub(super) layer_index: usize,
    pub(super) top_surfaces: &'a [Polygon],
    pub(super) supports_projected: &'a [Polygon],
    pub(super) top_contacts: &'a [TopContactLayer],
    pub(super) contact_index: i32,
    pub(super) bottom_interface_height: f64,
    pub(super) support_width_scaled: f32,
    pub(super) scale: CoordinateScale,
}

pub(super) fn detect_bottom_contacts(
    input: &BottomContactsInput<'_>,
    layer_support_areas: &mut [Vec<Polygon>],
) -> Result<Option<BottomContactLayer>, ClipperError> {
    if input.top_surfaces.is_empty() {
        return Ok(None);
    }
    let touching = intersection_polygons_paths(input.top_surfaces, input.supports_projected)?;
    if touching.is_empty() {
        return Ok(None);
    }
    let placement = input.placement;
    let object = placement.object;
    let parameters = placement.parameters;
    let layer = &placement.layers[input.layer_index];
    let zero_gap_bottom = parameters.zero_gap_interface_bottom;
    let (height, print_z) = if placement.independent_support_layer_height {
        if zero_gap_bottom {
            let upper = &placement.layers[input.layer_index + 1];
            (upper.height, upper.print_z)
        } else {
            (
                input.bottom_interface_height,
                layer.print_z + input.bottom_interface_height + parameters.gap_object_support,
            )
        }
    } else {
        let upper = sync_bottom_gap(
            placement.layers,
            input.layer_index,
            parameters.gap_object_support,
        );
        (upper.height, upper.print_z)
    };
    let mut contact = BottomContactLayer {
        print_z,
        bottom_z: layer.print_z,
        height,
        bridging: !zero_gap_bottom && object.thick_bridges.0,
        polygons: offset_paths(&touching, input.support_width_scaled, JoinType::Square, 0.0)?,
    };
    if !zero_gap_bottom {
        snap_to_top_contact(
            &mut contact,
            input.top_contacts,
            input.contact_index.max(0) as usize,
            placement.support_layer_height_min,
        );
    }
    let touching = offset_paths(
        &touching,
        (EPSILON / input.scale.factor()) as f32,
        JoinType::Miter,
        3.0,
    )?;
    for (index, above) in layer_support_areas
        .iter_mut()
        .enumerate()
        .skip(input.layer_index + 1)
    {
        if placement.layers[index].print_z > contact.print_z - EPSILON {
            break;
        }
        if !above.is_empty() {
            *above = difference_polygons_paths(above, &touching)?;
        }
    }
    Ok(Some(contact))
}

fn snap_to_top_contact(
    bottom: &mut BottomContactLayer,
    tops: &[TopContactLayer],
    start: usize,
    minimum_height: f64,
) {
    for top in &tops[start..] {
        if top.print_z >= bottom.print_z + minimum_height + EPSILON {
            break;
        }
        if top.print_z <= bottom.print_z - minimum_height - EPSILON {
            continue;
        }
        let difference = bottom.print_z - top.print_z;
        if difference > 0.0 && bottom.height - difference <= minimum_height {
            continue;
        }
        bottom.print_z = top.print_z;
        bottom.height -= difference;
        break;
    }
}

fn sync_bottom_gap(layers: &[PlannedLayer], layer_index: usize, gap: f64) -> &PlannedLayer {
    let mut gap_synced = 0.0_f32;
    let mut upper = layer_index + 1;
    let mut last_valid = upper;
    while upper < layers.len() && f64::from(gap_synced) < gap {
        last_valid = upper;
        gap_synced = (f64::from(gap_synced) + layers[upper].height) as f32;
        upper += 1;
    }
    if (f64::from(gap_synced) - layers[last_valid].height - gap).abs()
        < (f64::from(gap_synced) - gap).abs()
    {
        last_valid -= 1;
    }
    if gap > 0.0 {
        upper = if last_valid + 1 < layers.len() {
            last_valid + 1
        } else {
            last_valid
        };
    }
    &layers[upper]
}
