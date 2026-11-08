//! Ordinary grid base/flange generation from `SupportCommon.cpp:1764-1816`.

use super::{LayerCache, interfaces::SupportFillInput};
use crate::project_slice::{
    raft::{
        fill_params::{RaftFillPattern, tree_double_wall_area},
        fills::{RaftFillPreprocess, RaftFillSpec, raft_layer_entities},
    },
    support_material::{
        extrusion_path::SupportExtrusionEntity, grid_generation::GeneratedSupportGeometry,
    },
};
use crate::{ExtrusionRole, ProcessSupportBasePattern, ProcessSupportStyle, SliceError};

impl LayerCache<SupportExtrusionEntity> {
    pub(in crate::project_slice::support_material) fn fill_grid_base(
        &mut self,
        geometry: &mut GeneratedSupportGeometry,
        input: &SupportFillInput<'_>,
    ) -> Result<(), SliceError> {
        if let Some(base) = &mut self.base
            && !base.empty(geometry)
            && !base.polygons_to_extrude(geometry).is_empty()
        {
            let parameters = input.parameters;
            let first_layer = base.source.bottom_z(geometry) < 1e-4;
            let organic = !first_layer
                && crate::project_slice::support_material::style::resolve(
                    input.object.support_type,
                    input.object.support_style,
                ) == ProcessSupportStyle::Organic
                && matches!(
                    input.object.support_base_pattern,
                    ProcessSupportBasePattern::Hollow | ProcessSupportBasePattern::Default
                );
            let height = base.source.height_and_bridging(geometry).0 as f32;
            let flow = if first_layer {
                parameters.first_layer_flow
            } else {
                parameters.base_flow.with_height(height)?
            };
            let first_pattern = if input.range_begin == 0 {
                RaftFillPattern::Rectilinear
            } else {
                parameters.contact_fill_pattern
            };
            let spec = RaftFillSpec {
                angle: f64::from(if first_layer {
                    std::f32::consts::PI * (input.object.support_angle.0 + 90.0) as f32 / 180.0
                } else {
                    input.base_angle()
                }),
                spacing: f64::from(if first_layer || organic {
                    flow.spacing
                } else {
                    parameters.base_flow.spacing
                }),
                sheath_spacing: f64::from(flow.spacing),
                density: if first_layer {
                    (input.object.raft_first_layer_density.0 * 0.01) as f32
                } else if organic {
                    0.0
                } else {
                    parameters.support_density as f32
                },
                sheath: first_layer || organic || parameters.with_sheath,
                no_sort: first_layer,
                tree_double_wall_area: if organic && base.source.print_z(geometry) > 100.0 {
                    0.1
                } else {
                    tree_double_wall_area(
                        input.object.tree_support_wall_count.0,
                        1.0 / input.scale.factor(),
                    )
                },
                width_mm: f64::from(flow.width),
                pattern: if first_layer {
                    first_pattern
                } else {
                    parameters.base_fill_pattern
                },
                preprocess: RaftFillPreprocess::Closing,
            };
            let entities =
                raft_layer_entities(base.polygons_to_extrude(geometry), spec, input.scale)
                    .map_err(|error| {
                        SliceError::InvalidInput(format!("support base fill error: {error:?}"))
                    })?;
            base.extrusions.extend(entities.into_iter().map(|entity| {
                SupportExtrusionEntity::from_fill_entity(
                    entity,
                    ExtrusionRole::SupportMaterial,
                    flow,
                )
            }));
        }
        if let (Some(base), Some(base_interface)) = (&mut self.base, &mut self.base_interface)
            && !base.polygons_to_extrude(geometry).is_empty()
            && !base_interface.polygons_to_extrude(geometry).is_empty()
            && base.could_merge(base_interface, geometry)
        {
            base.merge(base_interface, geometry).map_err(|error| {
                SliceError::InvalidInput(format!("support base-interface merge error: {error:?}"))
            })?;
        }
        Ok(())
    }
}
