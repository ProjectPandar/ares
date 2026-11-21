//! Ordinary grid contact-cache preparation from `SupportCommon.cpp:1619-1658`.

use super::LayerCache;
use crate::project_slice::support_material::{
    extruded_layer::{ExtrudedLayer, SourceLayer},
    extrusion_path::SupportExtrusionEntity,
    grid_generation::GeneratedSupportGeometry,
    interface_counts::InterfaceCounts,
    interface_loops::LoopInterfaceProcessor,
    intermediate_heights::SupportLayerType,
    toolpath_parameters::ToolpathParameters,
};
use crate::{
    ObjectOptions, ProcessSupportStyle, SliceError,
    geometry::{CoordinateScale, JoinType, Polygon, difference_polygons_paths, offset_paths},
    project_slice::parameters::SlicingParameters,
};

pub(in crate::project_slice::support_material) struct ContactCacheInput<'a> {
    pub(in crate::project_slice::support_material) object: &'a ObjectOptions,
    pub(in crate::project_slice::support_material) slicing: &'a SlicingParameters,
    pub(in crate::project_slice::support_material) parameters: &'a ToolpathParameters,
    pub(in crate::project_slice::support_material) counts: &'a InterfaceCounts,
    pub(in crate::project_slice::support_material) loops: &'a LoopInterfaceProcessor,
    pub(in crate::project_slice::support_material) interface_angle: f32,
    pub(in crate::project_slice::support_material) scale: CoordinateScale,
}

pub(in crate::project_slice::support_material) struct IroningSurface {
    pub(in crate::project_slice::support_material) polygons: Vec<Polygon>,
    pub(in crate::project_slice::support_material) angle: f32,
}

pub(in crate::project_slice::support_material) struct ContactCacheState {
    pub(in crate::project_slice::support_material) raft_contact: bool,
    pub(in crate::project_slice::support_material) ironing: Option<IroningSurface>,
}

impl LayerCache<SupportExtrusionEntity> {
    pub(in crate::project_slice::support_material) fn prepare_grid_contacts(
        &mut self,
        geometry: &mut GeneratedSupportGeometry,
        input: ContactCacheInput<'_>,
    ) -> Result<ContactCacheState, SliceError> {
        let top_index = self.top.as_ref().map(|top| {
            let SourceLayer::Top(index) = top.source else {
                unreachable!("selected top-contact cache has a top-contact source");
            };
            index
        });
        let raft_contact = input.slicing.interface_raft_layers != 0
            && top_index.is_some_and(|index| {
                (geometry.tops[index].print_z - input.slicing.raft_contact_top_z).abs() < 1e-4
            });
        let top_zero = input.object.support_interface_top_layers.0 == 0;
        let organic = crate::project_slice::support_material::style::resolve(
            input.object.support_type,
            input.object.support_style,
        ) == ProcessSupportStyle::Organic;
        let bottom_zero = input.object.support_interface_bottom_layers.0 == 0;
        let mut ironing = None;
        if top_zero {
            if input.parameters.can_merge_support_regions
                && !raft_contact
                && !merge_if_compatible(&mut self.base, &mut self.top, geometry)?
                && empty(&self.base, geometry)
            {
                self.base = self.top.take();
            }
        } else {
            if input.object.support_ironing.0
                && let Some(top) = &self.top
                && !top.empty(geometry)
            {
                ironing = Some(IroningSurface {
                    polygons: top.polygons_to_extrude(geometry).to_vec(),
                    angle: input.interface_angle,
                });
            }
            if let Some(top) = &mut self.top
                && let Some(mut loops) = input.loops.extrude(
                    &mut geometry.tops
                        [top_index.expect("top cache and index are selected together")],
                    input.parameters.interface_flow,
                    input.scale,
                )?
            {
                top.set_polygons_to_extrude(loops.remaining);
                top.extrusions.append(&mut loops.paths);
            }
            if !raft_contact {
                merge_if_compatible(&mut self.top, &mut self.interface, geometry)?;
            }
        }
        if (top_zero || bottom_zero) && input.parameters.can_merge_support_regions {
            if !merge_if_compatible(&mut self.base, &mut self.bottom, geometry)?
                && empty(&self.base, geometry)
                && self.bottom.as_ref().is_some_and(|bottom| {
                    !bottom.empty(geometry) && !bottom.source.height_and_bridging(geometry).1
                })
            {
                self.base = self.bottom.take();
            }
        } else if !organic
            && (raft_contact || !merge_if_compatible(&mut self.top, &mut self.bottom, geometry)?)
            && let (Some(bottom), Some(interface)) = (&mut self.bottom, &mut self.interface)
            && bottom.could_merge(interface, geometry)
        {
            resolve_bottom_interface_overlap(bottom, interface, geometry, input.counts.bottom > 0)?;
        } else if organic && !raft_contact {
            merge_if_compatible(&mut self.top, &mut self.bottom, geometry)?;
        }
        if organic
            && input.slicing.zero_gap_interface_top
            && !input.parameters.can_merge_support_regions
            && (!empty(&self.base, geometry) || !empty(&self.base_interface, geometry))
        {
            self.trim_organic_interfaces(geometry, &input)?;
        }
        Ok(ContactCacheState {
            raft_contact,
            ironing,
        })
    }

    fn trim_organic_interfaces(
        &mut self,
        geometry: &GeneratedSupportGeometry,
        input: &ContactCacheInput<'_>,
    ) -> Result<(), SliceError> {
        let mut polygons = Vec::new();
        for layer in [&self.top, &self.interface]
            .into_iter()
            .flatten()
            .filter(|layer| !layer.empty(geometry))
        {
            polygons.extend_from_slice(layer.polygons_to_extrude(geometry));
        }
        if polygons.is_empty() {
            return Ok(());
        }
        let width = input
            .parameters
            .base_flow
            .width
            .max(input.parameters.interface_flow.width);
        let margin = input.scale.scaled_delta(f64::from(width)) as i64;
        let keepout =
            offset_paths(&polygons, margin as f32, JoinType::Miter, 3.0).map_err(|error| {
                SliceError::InvalidInput(format!("organic interface keepout: {error:?}"))
            })?;
        for layer in [&mut self.base, &mut self.base_interface]
            .into_iter()
            .flatten()
            .filter(|layer| !layer.empty(geometry))
        {
            let remaining =
                difference_polygons_paths(layer.polygons_to_extrude(geometry), &keepout).map_err(
                    |error| SliceError::InvalidInput(format!("organic interface trim: {error:?}")),
                )?;
            layer.set_polygons_to_extrude(remaining);
        }
        Ok(())
    }
}

fn resolve_bottom_interface_overlap<E>(
    bottom: &mut ExtrudedLayer<E>,
    interface: &mut ExtrudedLayer<E>,
    geometry: &mut GeneratedSupportGeometry,
    bottom_interfaces: bool,
) -> Result<(), SliceError> {
    let is_bottom_interface = matches!(interface.source, SourceLayer::Interface(index)
        if geometry.interfaces[index].layer_type == SupportLayerType::BottomInterface);
    // SupportCommon.cpp:1805-1812: only the merge updates the source polygons.
    if bottom_interfaces && is_bottom_interface {
        return bottom.merge(interface, geometry).map_err(|error| {
            SliceError::InvalidInput(format!("support contact merge error: {error:?}"))
        });
    }
    let remaining = difference_polygons_paths(
        bottom.polygons_to_extrude(geometry),
        interface.polygons_to_extrude(geometry),
    )
    .map_err(|error| {
        SliceError::InvalidInput(format!("support contact difference error: {error:?}"))
    })?;
    bottom.set_polygons_to_extrude(remaining);
    Ok(())
}

fn empty<E>(layer: &Option<ExtrudedLayer<E>>, geometry: &GeneratedSupportGeometry) -> bool {
    layer.as_ref().is_none_or(|layer| layer.empty(geometry))
}

fn merge_if_compatible<E>(
    destination: &mut Option<ExtrudedLayer<E>>,
    source: &mut Option<ExtrudedLayer<E>>,
    geometry: &mut GeneratedSupportGeometry,
) -> Result<bool, SliceError> {
    let (Some(destination), Some(source)) = (destination, source) else {
        return Ok(false);
    };
    if !destination.could_merge(source, geometry) {
        return Ok(false);
    }
    destination.merge(source, geometry).map_err(|error| {
        SliceError::InvalidInput(format!("support contact merge error: {error:?}"))
    })?;
    Ok(true)
}
