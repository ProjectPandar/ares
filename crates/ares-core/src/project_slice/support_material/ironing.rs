//! `SupportCommon.cpp:1881-1914` support-contact ironing.

use super::{
    extrusion_path::SupportExtrusionEntity, install_layers::InstalledSupportLayer,
    toolpath_cache::contacts::IroningSurface, toolpath_parameters::ToolpathParameters,
};
use crate::{
    ExtrusionRole, ObjectOptions, ProcessInfillPattern, SliceError,
    fill::rectilinear::{MonotonicFillParams, fill_rectilinear_surface},
    geometry::{CoordinateScale, Point, difference_ex, union_safety_offset_ex},
    project_slice::raft::fills::concentric,
};

pub(super) struct IroningInput<'a> {
    pub(super) object: &'a ObjectOptions,
    pub(super) parameters: &'a ToolpathParameters,
    pub(super) layer_id: usize,
    pub(super) print_z: f64,
    pub(super) upper: Option<&'a InstalledSupportLayer>,
    pub(super) scale: CoordinateScale,
}

impl IroningSurface {
    pub(super) fn append_paths(
        self,
        output: &mut Vec<SupportExtrusionEntity>,
        input: &IroningInput<'_>,
    ) -> Result<(), SliceError> {
        if self.polygons.is_empty() {
            return Ok(());
        }
        let mut regions = union_safety_offset_ex(&self.polygons).map_err(|error| {
            SliceError::InvalidInput(format!("support ironing union: {error:?}"))
        })?;
        if let Some(upper) = input.upper
            && !upper.support_islands.is_empty()
            && upper.print_z - upper.height <= input.print_z + 1e-4
        {
            regions = difference_ex(&regions, &upper.support_islands).map_err(|error| {
                SliceError::InvalidInput(format!("support ironing upper trim: {error:?}"))
            })?;
        }
        let spacing = f64::from(input.object.support_ironing_spacing.0 as f32);
        let height = (f64::from(input.parameters.raft_interface_flow.height)
            * 0.01
            * input.object.support_ironing_flow.0) as f32;
        for region in regions {
            let paths = match input.object.support_ironing_pattern {
                ProcessInfillPattern::Rectilinear => {
                    let parameters = MonotonicFillParams {
                        spacing,
                        overlap: 0.0,
                        density: 1.0,
                        angle: self.angle,
                        layer_index: input.layer_id,
                        thickness_layers: 1,
                        fixed_angle: false,
                        bridge_angle: None,
                        reference_point: Point::new(0, 0),
                        dont_adjust: true,
                        anchor_length_max: 1000.0,
                        link_max_length: 3.0 * spacing,
                        consistent_pattern: false,
                        horiz_move: 0.0,
                        extended_object_min_x: None,
                        extended_object_max_x: None,
                        locked_zag: false,
                    };
                    fill_rectilinear_surface(&region, parameters, input.scale)
                        .map(|filled| filled.polylines)
                }
                ProcessInfillPattern::Concentric => concentric(&region, spacing, 1.0, input.scale),
                _ => {
                    return Err(SliceError::UnsupportedProjectFeature(
                        "support_ironing_pattern".into(),
                    ));
                }
            }
            .map_err(|error| {
                SliceError::InvalidInput(format!("support ironing fill: {error:?}"))
            })?;
            // Native evaluates Flow::mm3_per_mm after filling each component.
            let flow = input.parameters.raft_interface_flow.with_height(height)?;
            SupportExtrusionEntity::append_paths(output, paths, ExtrusionRole::Ironing, flow);
        }
        Ok(())
    }
}
