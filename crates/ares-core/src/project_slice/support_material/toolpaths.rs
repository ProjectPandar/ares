//! Ordinary grid `SupportCommon.cpp::generate_support_toolpaths` driver.

mod raft;

use super::{
    grid_generation::GeneratedSupportGeometry,
    install_layers::InstalledSupportLayer,
    interface_counts::InterfaceCounts,
    interface_loops::LoopInterfaceProcessor,
    ironing::IroningInput,
    toolpath_cache::{
        LayerCacheIndices, contacts::ContactCacheInput, interfaces::SupportFillInput,
    },
    toolpath_parameters::ToolpathParameters,
};
use crate::{
    ObjectOptions, SliceError, SlicerContext, geometry::CoordinateScale,
    project_slice::parameters::SlicingParameters,
};

pub(super) struct ToolpathInput<'a> {
    pub(super) object: &'a ObjectOptions,
    pub(super) slicing: &'a SlicingParameters,
    pub(super) parameters: &'a ToolpathParameters,
    pub(super) counts: &'a InterfaceCounts,
    pub(super) scale: CoordinateScale,
}

pub(super) fn generate_grid_support_toolpaths(
    geometry: &mut GeneratedSupportGeometry,
    input: &ToolpathInput<'_>,
    context: &mut SlicerContext,
) -> Result<Vec<InstalledSupportLayer>, SliceError> {
    let mut layers = std::mem::take(&mut geometry.support_layers);
    let n_raft_layers = layers.len().min(
        (input.slicing.base_raft_layers + input.slicing.interface_raft_layers).saturating_sub(1),
    );
    raft::fill_prefix(&mut layers[..n_raft_layers], geometry, input, context)?;
    let width_scaled =
        (f64::from(input.parameters.interface_flow.width) / input.scale.factor()) as i64;
    let loops = LoopInterfaceProcessor::new(
        input.object.support_interface_loop_pattern.0,
        1.5 * width_scaled as f64,
    );
    let mut indices = LayerCacheIndices::default();
    let mut caches = Vec::with_capacity(layers.len() - n_raft_layers);
    for (index, layer) in layers.iter_mut().enumerate().skip(n_raft_layers) {
        let mut cache = indices.select(geometry, layer.print_z);
        let interface_angle = input
            .parameters
            .support_interface_angle(input.object, layer.interface_id);
        let contact = cache.prepare_grid_contacts(
            geometry,
            ContactCacheInput {
                object: input.object,
                slicing: input.slicing,
                parameters: input.parameters,
                loops: &loops,
                interface_angle,
                scale: input.scale,
            },
        )?;
        let fill = SupportFillInput {
            object: input.object,
            parameters: input.parameters,
            counts: input.counts,
            support_layer: layer,
            support_layer_index: index,
            range_begin: n_raft_layers,
            n_raft_layers,
            raft_contact: contact.raft_contact,
            interface_angle,
            scale: input.scale,
        };
        cache.fill_grid_interfaces(geometry, &fill, context)?;
        cache.fill_grid_base(geometry, &fill, context)?;
        let mut ordered = cache.collect_ordered(geometry, &indices).map_err(|error| {
            SliceError::InvalidInput(format!("support toolpath islands: {error:?}"))
        })?;
        layer.support_islands = std::mem::take(&mut ordered.islands);
        caches.push((ordered, contact.ironing));
    }
    // All islands are available before modulation and immediate-upper ironing trim.
    for (index, (mut cache, ironing)) in caches.into_iter().enumerate() {
        let index = index + n_raft_layers;
        let (lower, upper) = layers.split_at_mut(index + 1);
        let layer = &mut lower[index];
        cache.modulate_and_append(&mut layer.support_fills, geometry, input.scale)?;
        if let Some(ironing) = ironing {
            ironing.append_paths(
                &mut layer.support_fills,
                &IroningInput {
                    object: input.object,
                    parameters: input.parameters,
                    layer_id: layer.id,
                    print_z: layer.print_z,
                    height: layer.height,
                    upper: upper.first(),
                    scale: input.scale,
                },
            )?;
        }
    }
    Ok(layers)
}
