//! Non-multilayer-raft branch of `SupportCommon.cpp::generate_raft_base`.

use super::{contact_layer::TopContactLayer, intermediate_heights::IntermediateLayer};
use crate::geometry::{
    ClipperError, CoordinateScale, ExPolygon, JoinType, Polygon, difference_polygons_paths,
    offset_expolygons_paths, offset_paths,
};

pub(super) struct FirstLayerInput<'a> {
    pub(super) first_print_layer_height: f64,
    pub(super) raft_contact_top_z: f64,
    pub(super) raft_interface_top_z: f64,
    pub(super) first_object_slices: &'a [ExPolygon],
    // Brim trimming polygons from generate_raft_base's preceding brim branch.
    pub(super) brim: &'a [Polygon],
    pub(super) first_layer_expansion: f64,
    pub(super) gap_xy_first_layer: f64,
    pub(super) first_layer_flow_width_scaled: i64,
    pub(super) scale: CoordinateScale,
}

pub(super) fn expand_support_first_layer(
    input: &FirstLayerInput<'_>,
    tops: &mut [TopContactLayer],
    interfaces: &mut [IntermediateLayer],
    base_interfaces: &mut [IntermediateLayer],
    bases: &mut [IntermediateLayer],
) -> Result<(), ClipperError> {
    const EPSILON: f64 = 1e-4;
    let fine = (EPSILON / input.scale.factor()) as f32;
    let expansion = ((input.first_layer_expansion / input.scale.factor()) as f32 - fine).max(0.0);
    let contact_limit = input.first_print_layer_height.max(input.raft_contact_top_z) + EPSILON;
    let contacts = tops
        .first_mut()
        .filter(|layer| layer.print_z <= contact_limit);
    let interfaces = first_columns(interfaces, input.raft_interface_top_z);
    let base_interfaces = first_columns(base_interfaces, input.raft_interface_top_z);
    let columns = first_columns(bases, input.raft_interface_top_z);
    let mut interface_polygons = Vec::new();
    for polygons in [
        contacts.as_ref().map(|layer| layer.polygons.as_slice()),
        interfaces.as_ref().map(|layer| layer.polygons.as_slice()),
        base_interfaces
            .as_ref()
            .map(|layer| layer.polygons.as_slice()),
    ]
    .into_iter()
    .flatten()
    .filter(|polygons| !polygons.is_empty())
    {
        interface_polygons.extend(offset_paths(polygons, fine, JoinType::Square, 0.0)?);
    }
    if let Some(layer) = columns {
        let trimming = offset_expolygons_paths(
            input.first_object_slices,
            (input.gap_xy_first_layer / input.scale.factor()) as f32,
            JoinType::Square,
            0.0,
        )?;
        if f64::from(expansion) > EPSILON / input.scale.factor() {
            let steps =
                ((expansion / input.first_layer_flow_width_scaled as f32).ceil() as i32).max(5);
            let step = expansion / steps as f32;
            for _ in 0..steps {
                layer.polygons = difference_polygons_paths(
                    &offset_paths(&layer.polygons, step, JoinType::Miter, 3.0)?,
                    &trimming,
                )?;
            }
        } else {
            layer.polygons = difference_polygons_paths(&layer.polygons, &trimming)?;
        }
        if !interface_polygons.is_empty() {
            layer.polygons = difference_polygons_paths(&layer.polygons, &interface_polygons)?;
        }
        if !input.brim.is_empty() {
            layer.polygons = difference_polygons_paths(&layer.polygons, input.brim)?;
        }
    }
    if !input.brim.is_empty() {
        for polygons in [
            contacts.map(|layer| &mut layer.polygons),
            interfaces.map(|layer| &mut layer.polygons),
            base_interfaces.map(|layer| &mut layer.polygons),
        ]
        .into_iter()
        .flatten()
        {
            *polygons = difference_polygons_paths(polygons, input.brim)?;
        }
    }
    Ok(())
}

fn first_columns(
    layers: &mut [IntermediateLayer],
    interface_top_z: f64,
) -> Option<&mut IntermediateLayer> {
    layers
        .first_mut()
        .filter(|layer| layer.print_z - layer.height <= interface_top_z + 1e-4)
}
