//! Toolpath parameter slice of `SupportParameters.hpp:64-144`.

use crate::project_slice::{
    parameters::SlicingParameters,
    perimeters::{flow::SupportFlows, types::Flow},
    raft::fill_params::RaftFillPattern,
};
use crate::{
    ObjectOptions, ProcessSupportBasePattern, ProcessSupportInterfacePattern, ProcessSupportStyle,
};
use std::f32::consts::FRAC_PI_4;

pub(super) struct ToolpathParameters {
    pub(super) first_layer_flow: Flow,
    pub(super) base_flow: Flow,
    pub(super) interface_flow: Flow,
    pub(super) raft_interface_flow: Flow,
    pub(super) raft_interface_density: f64,
    pub(super) raft_interface_fill_pattern: RaftFillPattern,
    raft_angle_interface: f32,
    pub(super) raft_angle_first_layer: f32,
    pub(super) raft_angle_base: f32,
    pub(super) can_merge_support_regions: bool,
    pub(super) base_angle: f32,
    pub(super) interface_angle: f32,
    pub(super) top_interface_density: f64,
    pub(super) bottom_interface_density: f64,
    pub(super) support_density: f64,
    pub(super) with_sheath: bool,
    pub(super) base_fill_pattern: RaftFillPattern,
    pub(super) interface_fill_pattern: RaftFillPattern,
    pub(super) contact_fill_pattern: RaftFillPattern,
}

impl ToolpathParameters {
    pub(super) fn new(
        object: &ObjectOptions,
        slicing: &SlicingParameters,
        flows: &SupportFlows,
        object_extruders: &[usize],
    ) -> Result<Self, crate::SliceError> {
        let base_flow = flows.base;
        let interface_flow = flows.interface;
        let raft_interface_flow = interface_flow;
        let raft_spacing = (if object.support_ironing.0 {
            0.0
        } else {
            object.support_interface_spacing.0
        }) + f64::from(raft_interface_flow.spacing);
        let raft_interface_density =
            (f64::from(raft_interface_flow.spacing) / raft_spacing).min(1.0);
        let raft_interface_fill_pattern = if raft_interface_density > 0.95 {
            RaftFillPattern::Rectilinear
        } else {
            RaftFillPattern::SupportBase
        };
        let interface_flow = if object.support_interface_top_layers.0 == 0 {
            base_flow
        } else {
            interface_flow
        };
        let base_filament = object.support_filament.0;
        let interface_filament = object.support_interface_filament.0;
        let can_merge_support_regions = base_filament == interface_filament
            || ((base_filament == 0 || interface_filament == 0)
                && object_extruders.len() == 1
                // Upstream compares zero-based object extruders directly to selectors.
                && object_extruders[0] == base_filament.max(interface_filament) as usize);
        let base_angle = std::f32::consts::PI * object.support_angle.0 as f32 / 180.0;
        let interface_angle = std::f32::consts::PI * (object.support_angle.0 + 90.0) as f32 / 180.0;
        let raft_angle_interface = if slicing.base_raft_layers > 1 {
            if slicing.interface_raft_layers.is_multiple_of(2) {
                interface_angle + (0.5 * std::f64::consts::PI) as f32
            } else {
                interface_angle
            }
        } else if slicing.base_raft_layers == 1 || slicing.interface_raft_layers > 1 {
            (f64::from(interface_angle) + 0.5 * std::f64::consts::PI) as f32
        } else if slicing.interface_raft_layers == 1 {
            (0.5 * std::f64::consts::PI) as f32
        } else {
            0.0
        };
        let (raft_angle_first_layer, raft_angle_base) = if slicing.base_raft_layers > 1 {
            (interface_angle, base_angle)
        } else if slicing.base_raft_layers == 1 || slicing.interface_raft_layers > 1 {
            (base_angle, 0.0)
        } else if slicing.interface_raft_layers == 1 {
            ((0.5 * std::f64::consts::PI) as f32, 0.0)
        } else {
            (0.0, 0.0)
        };
        let top_spacing = if object.support_ironing.0 {
            0.0
        } else {
            object.support_interface_spacing.0
        } + f64::from(interface_flow.spacing);
        let bottom_spacing =
            object.support_bottom_interface_spacing.0 + f64::from(interface_flow.spacing);
        let mut top_interface_density = (f64::from(interface_flow.spacing) / top_spacing).min(1.0);
        let bottom_interface_density =
            (f64::from(interface_flow.spacing) / bottom_spacing).min(1.0);
        let support_density = (f64::from(base_flow.spacing)
            / (object.support_base_pattern_spacing.0 + f64::from(base_flow.spacing)))
        .min(1.0);
        if object.support_interface_top_layers.0 == 0 {
            top_interface_density = support_density;
        }
        let with_sheath = object.tree_support_wall_count.0 > 0;
        let base_fill_pattern =
            if object.support_base_pattern == ProcessSupportBasePattern::Honeycomb {
                RaftFillPattern::Honeycomb
            } else if support_density > 0.95 || with_sheath {
                RaftFillPattern::Rectilinear
            } else {
                RaftFillPattern::SupportBase
            };
        let interface_fill_pattern = if top_interface_density > 0.95 {
            RaftFillPattern::Rectilinear
        } else {
            RaftFillPattern::SupportBase
        };
        let contact_fill_pattern = match object.support_interface_pattern {
            ProcessSupportInterfacePattern::Grid => RaftFillPattern::Grid,
            ProcessSupportInterfacePattern::RectilinearInterlaced => RaftFillPattern::Rectilinear,
            ProcessSupportInterfacePattern::Concentric => RaftFillPattern::Concentric,
            ProcessSupportInterfacePattern::SpiralInset => RaftFillPattern::SpiralInset,
            ProcessSupportInterfacePattern::Auto if slicing.zero_gap_interface_top => {
                RaftFillPattern::Concentric
            }
            ProcessSupportInterfacePattern::Auto | ProcessSupportInterfacePattern::Rectilinear => {
                interface_fill_pattern
            }
        };
        Ok(Self {
            first_layer_flow: flows.first_layer,
            base_flow,
            interface_flow,
            raft_interface_flow,
            raft_interface_density,
            raft_interface_fill_pattern,
            raft_angle_interface,
            raft_angle_first_layer,
            raft_angle_base,
            can_merge_support_regions,
            base_angle,
            interface_angle,
            top_interface_density,
            bottom_interface_density,
            support_density,
            with_sheath,
            base_fill_pattern,
            interface_fill_pattern,
            contact_fill_pattern,
        })
    }

    /// `SupportParameters.hpp:308–329::support_interface_angle`.
    pub(super) fn support_interface_angle(
        &self,
        object: &ObjectOptions,
        interface_id: usize,
    ) -> f32 {
        match object.support_interface_pattern {
            ProcessSupportInterfacePattern::Rectilinear => {
                if super::style::resolve(object.support_type, object.support_style)
                    == ProcessSupportStyle::Snug
                {
                    self.interface_angle - FRAC_PI_4
                } else {
                    self.interface_angle
                }
            }
            ProcessSupportInterfacePattern::RectilinearInterlaced => {
                self.interface_angle
                    + if interface_id & 1 != 0 {
                        FRAC_PI_4
                    } else {
                        -FRAC_PI_4
                    }
            }
            ProcessSupportInterfacePattern::Grid => self.base_angle,
            ProcessSupportInterfacePattern::Auto
            | ProcessSupportInterfacePattern::Concentric
            | ProcessSupportInterfacePattern::SpiralInset => self.interface_angle,
        }
    }

    pub(super) fn raft_interface_angle(&self, interface_id: usize) -> f32 {
        self.raft_angle_interface
            + if interface_id & 1 == 0 {
                (std::f64::consts::PI / 4.0) as f32
            } else {
                (-std::f64::consts::PI / 4.0) as f32
            }
    }
}
