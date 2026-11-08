//! `SupportCommon.cpp:1693-1762` ordinary grid interface extrusion calls.

use super::LayerCache;
use crate::geometry::{CoordinateScale, Point};
use crate::project_slice::{
    perimeters::types::Flow,
    raft::{
        fill_params::{RaftFillPattern, tree_double_wall_area},
        fills::{RaftFillPreprocess, RaftFillSpec, raft_layer_fill},
    },
    support_material::{
        extruded_layer::{ExtrudedLayer, SourceLayer},
        extrusion_path::SupportExtrusionEntity,
        grid_generation::GeneratedSupportGeometry,
        install_layers::InstalledSupportLayer,
        interface_counts::InterfaceCounts,
        intermediate_heights::SupportLayerType,
        toolpath_parameters::ToolpathParameters,
    },
};
use crate::{
    ExtrusionRole, ObjectOptions, ProcessSupportBasePattern, ProcessSupportStyle, SliceError,
};

pub(in crate::project_slice::support_material) struct SupportFillInput<'a> {
    pub(in crate::project_slice::support_material) object: &'a ObjectOptions,
    pub(in crate::project_slice::support_material) parameters: &'a ToolpathParameters,
    pub(in crate::project_slice::support_material) counts: &'a InterfaceCounts,
    pub(in crate::project_slice::support_material) support_layer: &'a InstalledSupportLayer,
    pub(in crate::project_slice::support_material) support_layer_index: usize,
    pub(in crate::project_slice::support_material) range_begin: usize,
    pub(in crate::project_slice::support_material) n_raft_layers: usize,
    pub(in crate::project_slice::support_material) raft_contact: bool,
    pub(in crate::project_slice::support_material) interface_angle: f32,
    pub(in crate::project_slice::support_material) scale: CoordinateScale,
}

enum InterfaceLayerType {
    TopContact,
    BottomContact,
    RaftContact,
    Interface,
    InterfaceAsBase,
}

struct InterfaceFillPlan {
    flow: Flow,
    role: ExtrusionRole,
    angle: f32,
    density: f64,
    spacing: f32,
    pattern: RaftFillPattern,
}

impl LayerCache<SupportExtrusionEntity> {
    pub(in crate::project_slice::support_material) fn fill_grid_interfaces(
        &mut self,
        geometry: &GeneratedSupportGeometry,
        input: &SupportFillInput<'_>,
    ) -> Result<(), SliceError> {
        let top_interfaces = input.counts.top > 0;
        let bottom_interfaces = top_interfaces && input.counts.bottom > 0;
        let top_type = if input.raft_contact {
            InterfaceLayerType::RaftContact
        } else if top_interfaces {
            InterfaceLayerType::TopContact
        } else {
            InterfaceLayerType::InterfaceAsBase
        };
        extrude_interface(&mut self.top, geometry, top_type, input)?;
        let bottom_type = if bottom_interfaces {
            InterfaceLayerType::BottomContact
        } else {
            InterfaceLayerType::InterfaceAsBase
        };
        if crate::project_slice::support_material::style::resolve(
            input.object.support_type,
            input.object.support_style,
        ) != ProcessSupportStyle::Organic
        {
            extrude_interface(&mut self.bottom, geometry, bottom_type, input)?;
        }
        let interface_type = if top_interfaces {
            InterfaceLayerType::Interface
        } else {
            InterfaceLayerType::InterfaceAsBase
        };
        extrude_interface(&mut self.interface, geometry, interface_type, input)?;
        if let Some(base_interface) = &mut self.base_interface
            && !base_interface.empty(geometry)
            && !base_interface.polygons_to_extrude(geometry).is_empty()
        {
            let parameters = input.parameters;
            let height = base_interface.source.height_and_bridging(geometry).0 as f32;
            let plan = InterfaceFillPlan {
                flow: parameters.base_flow.with_height(height)?,
                role: ExtrusionRole::SupportMaterial,
                angle: input.interface_angle,
                density: parameters.top_interface_density,
                spacing: parameters.interface_flow.spacing,
                pattern: if parameters.top_interface_density > 0.95 || parameters.with_sheath {
                    RaftFillPattern::Rectilinear
                } else {
                    RaftFillPattern::SupportBase
                },
            };
            fill_layer(base_interface, geometry, plan, input)?;
        }
        Ok(())
    }
}

impl SupportFillInput<'_> {
    pub(super) fn base_angle(&self) -> f32 {
        if self.object.support_base_pattern == ProcessSupportBasePattern::RectilinearGrid
            && self.support_layer_index & 1 != 0
        {
            self.parameters.interface_angle
        } else {
            self.parameters.base_angle
        }
    }
}

fn extrude_interface(
    layer: &mut Option<ExtrudedLayer<SupportExtrusionEntity>>,
    geometry: &GeneratedSupportGeometry,
    kind: InterfaceLayerType,
    input: &SupportFillInput<'_>,
) -> Result<(), SliceError> {
    let Some(layer) = layer else {
        return Ok(());
    };
    if layer.empty(geometry) || layer.polygons_to_extrude(geometry).is_empty() {
        return Ok(());
    }
    let parameters = input.parameters;
    let as_base = matches!(kind, InterfaceLayerType::InterfaceAsBase);
    let raft = matches!(kind, InterfaceLayerType::RaftContact);
    let bottom = matches!(kind, InterfaceLayerType::BottomContact)
        || (matches!(kind, InterfaceLayerType::Interface)
            && matches!(layer.source, SourceLayer::Interface(index)
                if geometry.interfaces[index].layer_type == SupportLayerType::BottomInterface));
    let (height, bridging) = layer.source.height_and_bridging(geometry);
    let source_flow = if raft {
        parameters.raft_interface_flow
    } else if as_base {
        parameters.base_flow
    } else {
        parameters.interface_flow
    };
    // SupportParameters' bottom-flow transformations retain this nozzle diameter.
    let flow = if bridging {
        Flow::bridging_flow(height as f32, parameters.interface_flow.nozzle_diameter)
    } else {
        source_flow.with_height(height as f32)?
    };
    let plan = InterfaceFillPlan {
        flow,
        role: if as_base {
            ExtrusionRole::SupportMaterial
        } else {
            ExtrusionRole::SupportMaterialInterface
        },
        angle: if as_base {
            input.base_angle()
        } else if raft {
            parameters.raft_interface_angle(input.support_layer.interface_id)
        } else {
            input.interface_angle
        },
        density: if raft {
            parameters.raft_interface_density
        } else if as_base {
            parameters.support_density
        } else if bottom {
            parameters.bottom_interface_density
        } else {
            parameters.top_interface_density
        },
        spacing: source_flow.spacing,
        pattern: if raft
            && input.range_begin == input.n_raft_layers
            && input.object.support_interface_top_layers.0 == 0
        {
            parameters.raft_interface_fill_pattern
        } else {
            parameters.contact_fill_pattern
        },
    };
    fill_layer(layer, geometry, plan, input)
}

fn fill_layer(
    layer: &mut ExtrudedLayer<SupportExtrusionEntity>,
    geometry: &GeneratedSupportGeometry,
    plan: InterfaceFillPlan,
    input: &SupportFillInput<'_>,
) -> Result<(), SliceError> {
    let spec = RaftFillSpec {
        angle: f64::from(plan.angle),
        spacing: f64::from(plan.spacing),
        sheath_spacing: f64::from(plan.flow.spacing),
        density: plan.density as f32,
        sheath: false,
        no_sort: false,
        tree_double_wall_area: tree_double_wall_area(
            input.object.tree_support_wall_count.0,
            1.0 / input.scale.factor(),
        ),
        width_mm: f64::from(plan.flow.width),
        pattern: plan.pattern,
        preprocess: RaftFillPreprocess::SafetyOffset,
    };
    let lines = raft_layer_fill(
        layer.polygons_to_extrude(geometry),
        spec,
        Point::new(0, 0),
        input.scale,
    )
    .map_err(|error| {
        SliceError::InvalidInput(format!("support interface fill error: {error:?}"))
    })?;
    SupportExtrusionEntity::append_paths(&mut layer.extrusions, lines, plan.role, plan.flow);
    Ok(())
}
