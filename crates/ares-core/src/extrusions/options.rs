mod accessors;
mod hardware;

use super::SmallAreaInfillFlowCompensation;
pub(crate) use hardware::{
    AUTO_WIDTH_RATIO, ExtrusionWidthSpec, RoleExtrusionHardware, RoleHardwareValues,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ExplicitExtrusionSegment {
    pub(crate) role: crate::PrintPathRole,
    pub(crate) layer_height: f64,
    pub(crate) is_first_layer: bool,
    pub(crate) line_width: f64,
    pub(crate) line_length_mm: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ExtrusionOptions {
    hardware: RoleExtrusionHardware,
    support_material_extrusion_hardware: Option<RoleHardwareValues>,
    line_width: ExtrusionWidthSpec,
    initial_layer_line_width: f64,
    outer_wall_line_width: ExtrusionWidthSpec,
    inner_wall_line_width: ExtrusionWidthSpec,
    outer_wall_flow_ratio: f64,
    inner_wall_flow_ratio: f64,
    overhang_flow_ratio: f64,
    sparse_infill_line_width: ExtrusionWidthSpec,
    internal_solid_infill_line_width: ExtrusionWidthSpec,
    top_surface_line_width: ExtrusionWidthSpec,
    support_line_width: ExtrusionWidthSpec,
    bridge_flow: f64,
    thick_bridges: bool,
    thick_internal_bridges: bool,
    internal_bridge_flow: f64,
    brim_flow_ratio: f64,
    gap_fill_flow_ratio: f64,
    sparse_infill_flow_ratio: f64,
    internal_solid_infill_flow_ratio: f64,
    support_flow_ratio: f64,
    support_interface_flow_ratio: f64,
    top_solid_infill_flow_ratio: f64,
    ironing_flow_ratio: f64,
    bottom_solid_infill_flow_ratio: f64,
    first_layer_flow_ratio: f64,
    filament_flow_ratio: f64,
    print_flow_ratio: f64,
    small_area_infill_flow_compensation: SmallAreaInfillFlowCompensation,
}

impl ExtrusionOptions {
    pub const fn new_for_tests(
        nozzle_diameter: f64,
        filament_diameter: f64,
        line_width: f64,
        wall_line_widths: (f64, f64),
        sparse_infill_line_width: f64,
    ) -> Self {
        Self {
            hardware: RoleExtrusionHardware::first(nozzle_diameter, filament_diameter),
            support_material_extrusion_hardware: None,
            line_width: ExtrusionWidthSpec::absolute(line_width),
            initial_layer_line_width: 0.0,
            outer_wall_line_width: ExtrusionWidthSpec::absolute(wall_line_widths.0),
            inner_wall_line_width: ExtrusionWidthSpec::absolute(wall_line_widths.1),
            outer_wall_flow_ratio: 1.0,
            inner_wall_flow_ratio: 1.0,
            overhang_flow_ratio: 1.0,
            sparse_infill_line_width: ExtrusionWidthSpec::absolute(sparse_infill_line_width),
            internal_solid_infill_line_width: ExtrusionWidthSpec::auto(),
            top_surface_line_width: ExtrusionWidthSpec::auto(),
            support_line_width: ExtrusionWidthSpec::auto(),
            bridge_flow: 1.0,
            thick_bridges: false,
            thick_internal_bridges: false,
            internal_bridge_flow: 1.0,
            brim_flow_ratio: 1.0,
            gap_fill_flow_ratio: 1.0,
            sparse_infill_flow_ratio: 1.0,
            internal_solid_infill_flow_ratio: 1.0,
            support_flow_ratio: 1.0,
            support_interface_flow_ratio: 1.0,
            top_solid_infill_flow_ratio: 1.0,
            ironing_flow_ratio: 1.0,
            bottom_solid_infill_flow_ratio: 1.0,
            first_layer_flow_ratio: 1.0,
            filament_flow_ratio: 1.0,
            print_flow_ratio: 1.0,
            small_area_infill_flow_compensation: SmallAreaInfillFlowCompensation::disabled(),
        }
    }

    pub(crate) fn with_line_width_spec(&self, line_width: ExtrusionWidthSpec) -> Self {
        Self {
            line_width,
            ..self.clone()
        }
    }

    pub(crate) fn with_outer_wall_line_width_spec(
        &self,
        outer_wall_line_width: ExtrusionWidthSpec,
    ) -> Self {
        Self {
            outer_wall_line_width,
            ..self.clone()
        }
    }

    pub(crate) fn with_inner_wall_line_width_spec(
        &self,
        inner_wall_line_width: ExtrusionWidthSpec,
    ) -> Self {
        Self {
            inner_wall_line_width,
            ..self.clone()
        }
    }

    pub(crate) fn with_sparse_infill_line_width_spec(
        &self,
        sparse_infill_line_width: ExtrusionWidthSpec,
    ) -> Self {
        Self {
            sparse_infill_line_width,
            ..self.clone()
        }
    }

    pub(crate) fn with_internal_solid_infill_line_width_spec(
        &self,
        internal_solid_infill_line_width: ExtrusionWidthSpec,
    ) -> Self {
        Self {
            internal_solid_infill_line_width,
            ..self.clone()
        }
    }

    pub(crate) fn with_top_surface_line_width_spec(
        &self,
        top_surface_line_width: ExtrusionWidthSpec,
    ) -> Self {
        Self {
            top_surface_line_width,
            ..self.clone()
        }
    }

    pub(crate) fn with_role_hardware(&self, hardware: RoleExtrusionHardware) -> Self {
        Self {
            hardware,
            ..self.clone()
        }
    }

    pub(crate) fn with_support_material_extrusion_hardware(
        &self,
        support_material_extrusion_hardware: RoleHardwareValues,
    ) -> Self {
        Self {
            support_material_extrusion_hardware: Some(support_material_extrusion_hardware),
            ..self.clone()
        }
    }

    #[cfg(test)]
    pub(crate) fn with_role_hardware_for_tests(
        &self,
        wall: RoleHardwareValues,
        sparse_infill: RoleHardwareValues,
        solid_infill: RoleHardwareValues,
    ) -> Self {
        self.with_role_hardware(
            RoleExtrusionHardware::from_default(self.hardware.default)
                .with_wall(wall)
                .with_sparse_infill(sparse_infill)
                .with_solid_infill(solid_infill),
        )
    }

    #[cfg(test)]
    pub(crate) fn with_support_hardware_for_tests(
        &self,
        support: RoleHardwareValues,
        support_interface: RoleHardwareValues,
    ) -> Self {
        self.with_role_hardware(
            self.hardware
                .with_support(support)
                .with_support_interface(support_interface),
        )
    }

    pub fn with_bridge_flow(&self, bridge_flow: f64) -> Self {
        Self {
            bridge_flow,
            ..self.clone()
        }
    }

    pub fn with_thick_bridges(&self, thick_bridges: bool) -> Self {
        Self {
            thick_bridges,
            ..self.clone()
        }
    }

    pub fn with_thick_internal_bridges(&self, thick_internal_bridges: bool) -> Self {
        Self {
            thick_internal_bridges,
            ..self.clone()
        }
    }

    pub fn with_initial_layer_line_width(&self, initial_layer_line_width: f64) -> Self {
        Self {
            initial_layer_line_width,
            ..self.clone()
        }
    }

    pub fn with_internal_solid_infill_line_width(
        &self,
        internal_solid_infill_line_width: f64,
    ) -> Self {
        self.with_internal_solid_infill_line_width_spec(ExtrusionWidthSpec::absolute(
            internal_solid_infill_line_width,
        ))
    }

    pub fn with_top_surface_line_width(&self, top_surface_line_width: f64) -> Self {
        self.with_top_surface_line_width_spec(ExtrusionWidthSpec::absolute(top_surface_line_width))
    }

    pub(crate) fn with_support_line_width_spec(
        &self,
        support_line_width: ExtrusionWidthSpec,
    ) -> Self {
        Self {
            support_line_width,
            ..self.clone()
        }
    }

    pub fn with_support_line_width(&self, support_line_width: f64) -> Self {
        self.with_support_line_width_spec(ExtrusionWidthSpec::absolute(support_line_width))
    }

    pub fn with_internal_bridge_flow(&self, internal_bridge_flow: f64) -> Self {
        Self {
            internal_bridge_flow,
            ..self.clone()
        }
    }

    pub fn with_brim_flow_ratio(&self, brim_flow_ratio: f64) -> Self {
        Self {
            brim_flow_ratio,
            ..self.clone()
        }
    }

    pub fn with_gap_fill_flow_ratio(&self, gap_fill_flow_ratio: f64) -> Self {
        Self {
            gap_fill_flow_ratio,
            ..self.clone()
        }
    }

    pub fn with_outer_wall_flow_ratio(&self, outer_wall_flow_ratio: f64) -> Self {
        Self {
            outer_wall_flow_ratio,
            ..self.clone()
        }
    }

    pub fn with_inner_wall_flow_ratio(&self, inner_wall_flow_ratio: f64) -> Self {
        Self {
            inner_wall_flow_ratio,
            ..self.clone()
        }
    }

    pub fn with_overhang_flow_ratio(&self, overhang_flow_ratio: f64) -> Self {
        Self {
            overhang_flow_ratio,
            ..self.clone()
        }
    }

    pub fn with_sparse_infill_flow_ratio(&self, sparse_infill_flow_ratio: f64) -> Self {
        Self {
            sparse_infill_flow_ratio,
            ..self.clone()
        }
    }

    pub fn with_internal_solid_infill_flow_ratio(
        &self,
        internal_solid_infill_flow_ratio: f64,
    ) -> Self {
        Self {
            internal_solid_infill_flow_ratio,
            ..self.clone()
        }
    }

    pub fn with_support_flow_ratio(&self, support_flow_ratio: f64) -> Self {
        Self {
            support_flow_ratio,
            ..self.clone()
        }
    }

    pub fn with_support_interface_flow_ratio(&self, support_interface_flow_ratio: f64) -> Self {
        Self {
            support_interface_flow_ratio,
            ..self.clone()
        }
    }

    pub fn with_top_solid_infill_flow_ratio(&self, top_solid_infill_flow_ratio: f64) -> Self {
        Self {
            top_solid_infill_flow_ratio,
            ..self.clone()
        }
    }

    pub fn with_ironing_flow_ratio(&self, ironing_flow_ratio: f64) -> Self {
        Self {
            ironing_flow_ratio,
            ..self.clone()
        }
    }

    pub fn with_bottom_solid_infill_flow_ratio(&self, bottom_solid_infill_flow_ratio: f64) -> Self {
        Self {
            bottom_solid_infill_flow_ratio,
            ..self.clone()
        }
    }

    pub fn with_first_layer_flow_ratio(&self, first_layer_flow_ratio: f64) -> Self {
        Self {
            first_layer_flow_ratio,
            ..self.clone()
        }
    }

    pub fn with_print_flow_ratio(&self, print_flow_ratio: f64) -> Self {
        Self {
            print_flow_ratio,
            ..self.clone()
        }
    }

    pub fn with_filament_flow_ratio(&self, filament_flow_ratio: f64) -> Self {
        Self {
            filament_flow_ratio,
            ..self.clone()
        }
    }

    pub(crate) fn with_small_area_infill_flow_compensation(
        &self,
        small_area_infill_flow_compensation: SmallAreaInfillFlowCompensation,
    ) -> Self {
        Self {
            small_area_infill_flow_compensation,
            ..self.clone()
        }
    }
}
