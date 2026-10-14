//! `SupportCommon.cpp:1881-1914` support-contact ironing: pattern dispatch
//! goes through `fill_entities::surface` (the `FillBase.cpp:40-77` /
//! `SupportCommon.cpp:1879` `Fill::new_from_type` mirror shared with the
//! sparse fills), parameterized per `SupportCommon.cpp:1880-1888`.

use super::{
    extrusion_path::SupportExtrusionEntity, install_layers::InstalledSupportLayer,
    toolpath_cache::contacts::IroningSurface, toolpath_parameters::ToolpathParameters,
};
use crate::{
    ExtrusionRole, ObjectOptions,
    geometry::{CoordinateScale, difference_ex, union_safety_offset_ex},
    project_slice::{
        fill_entities::{SurfaceCall, SurfaceRequest, surface_polylines},
        group_fills::{SurfaceFillParams, SurfaceFillPattern},
    },
};

pub(super) struct IroningInput<'a> {
    pub(super) object: &'a ObjectOptions,
    pub(super) parameters: &'a ToolpathParameters,
    pub(super) layer_id: usize,
    pub(super) print_z: f64,
    pub(super) height: f64,
    pub(super) upper: Option<&'a InstalledSupportLayer>,
    pub(super) scale: CoordinateScale,
}

impl IroningSurface {
    pub(super) fn append_paths(
        self,
        output: &mut Vec<SupportExtrusionEntity>,
        input: &IroningInput<'_>,
    ) -> Result<(), crate::SliceError> {
        if self.polygons.is_empty() {
            return Ok(());
        }
        let mut regions = union_safety_offset_ex(&self.polygons).map_err(|error| {
            crate::SliceError::InvalidInput(format!("support ironing union: {error:?}"))
        })?;
        if let Some(upper) = input.upper
            && !upper.support_islands.is_empty()
            && upper.print_z - upper.height <= input.print_z + 1e-4
        {
            regions = difference_ex(&regions, &upper.support_islands).map_err(|error| {
                crate::SliceError::InvalidInput(format!("support ironing upper trim: {error:?}"))
            })?;
        }
        let spacing = f64::from(input.object.support_ironing_spacing.0 as f32);
        let height = (f64::from(input.parameters.raft_interface_flow.height)
            * 0.01
            * input.object.support_ironing_flow.0) as f32;
        // `SupportCommon.cpp:1880-1888`: overlap 0, angle = the interface
        // angle, spacing = ironing spacing, density full, `link_max_length =
        // 3 × spacing`; the wrapper fixes `dont_adjust`
        // (`SupportCommon.cpp:450-453`); anchors default to 1000
        // (`FillParams`, `FillBase.hpp:56-61`).
        let params = SurfaceFillParams {
            extruder: 0,
            pattern: SurfaceFillPattern::Configured(input.object.support_ironing_pattern),
            spacing,
            overlap: 0.0,
            angle: self.angle,
            fixed_angle: false,
            bridge: false,
            bridge_angle: 0.0,
            density: 100.0,
            multiline: 1,
            adaptive_line_spacing: 0.0,
            anchor_length: 1000.0,
            anchor_length_max: 1000.0,
            flow: input.parameters.raft_interface_flow,
            flow_ratio: 1.0,
            extrusion_role: ExtrusionRole::Ironing,
            idx: 0,
            loop_clipping: 0,
            role_speed: 0.0,
            lateral_lattice_angle_1: -45.0,
            lateral_lattice_angle_2: 45.0,
            infill_lock_depth: 1.0,
            skin_infill_depth: 2.0,
            symmetric_infill_y_axis: false,
            horiz_move: 0.0,
            infill_overhang_angle: 60.0,
            gyroid_optimized: false,
            filter_out_gap_fill: 0.0,
            gap_fill_target: crate::ProcessGapFillTarget::default(),
        };
        let call = SurfaceCall {
            dont_alternate_fill_direction: false,
            layer_id: input.layer_id,
            reference: crate::geometry::Point::new(0, 0),
            scale: input.scale,
            link_max_length: 3.0 * spacing,
            dont_adjust: true,
        };
        for region in regions {
            let request = SurfaceRequest {
                pattern: input.object.support_ironing_pattern,
                expolygon: region,
                params,
                thickness_layers: 1,
                layer_height: input.height,
                print_z: input.print_z,
                call,
            };
            let paths = surface_polylines(&request).map_err(|error| {
                crate::SliceError::InvalidInput(format!("support ironing fill: {error:?}"))
            })?;
            // Native evaluates Flow::mm3_per_mm after filling each component.
            let flow = input.parameters.raft_interface_flow.with_height(height)?;
            SupportExtrusionEntity::append_paths(output, paths, ExtrusionRole::Ironing, flow);
        }
        Ok(())
    }
}
