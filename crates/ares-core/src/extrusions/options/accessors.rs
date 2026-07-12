use super::{AUTO_WIDTH_RATIO, ExplicitExtrusionSegment, ExtrusionOptions, RoleHardwareValues};
use crate::{PrintPathRole, SliceError};

impl ExtrusionOptions {
    pub const fn filament_diameter(&self) -> f64 {
        self.hardware.default.filament_diameter
    }

    pub(crate) fn line_width_mm(&self) -> f64 {
        let nozzle_diameter = self.hardware.default.nozzle_diameter;
        let line_width = self.line_width.resolve(nozzle_diameter);
        if line_width > 0.0 {
            line_width
        } else {
            AUTO_WIDTH_RATIO * nozzle_diameter
        }
    }

    pub fn width_for_role(&self, role: PrintPathRole) -> f64 {
        let nozzle_diameter = self.nozzle_diameter_for_role(role);
        let line_width = self.line_width.resolve(nozzle_diameter);
        let role_width = match role {
            PrintPathRole::Skirt
            | PrintPathRole::Brim
            | PrintPathRole::Bridge
            | PrintPathRole::InternalBridge
            | PrintPathRole::GapFill => 0.0,
            PrintPathRole::ExternalPerimeter | PrintPathRole::OverhangPerimeter => {
                self.outer_wall_line_width.resolve(nozzle_diameter)
            }
            PrintPathRole::InternalPerimeter => self.inner_wall_line_width.resolve(nozzle_diameter),
            PrintPathRole::SparseInfill => self.sparse_infill_line_width.resolve(nozzle_diameter),
            PrintPathRole::SolidInfill | PrintPathRole::BottomSurface => self
                .internal_solid_infill_line_width
                .resolve(nozzle_diameter),
            PrintPathRole::SupportMaterial | PrintPathRole::SupportMaterialInterface => {
                self.support_line_width.resolve(nozzle_diameter)
            }
            PrintPathRole::TopSolidInfill | PrintPathRole::Ironing => {
                let top_surface_line_width = self.top_surface_line_width.resolve(nozzle_diameter);
                if top_surface_line_width > 0.0 {
                    return top_surface_line_width;
                }
                if line_width > 0.0 {
                    return line_width;
                }
                return nozzle_diameter;
            }
        };
        if role_width > 0.0 {
            role_width
        } else if line_width > 0.0 {
            line_width
        } else {
            AUTO_WIDTH_RATIO * nozzle_diameter
        }
    }

    pub fn width_for_role_and_layer(&self, role: PrintPathRole, is_first_layer: bool) -> f64 {
        match role {
            PrintPathRole::Skirt
            | PrintPathRole::Brim
            | PrintPathRole::ExternalPerimeter
            | PrintPathRole::OverhangPerimeter
            | PrintPathRole::InternalPerimeter
            | PrintPathRole::GapFill
            | PrintPathRole::SparseInfill
            | PrintPathRole::SolidInfill
            | PrintPathRole::TopSolidInfill
            | PrintPathRole::BottomSurface
            | PrintPathRole::SupportMaterial
            | PrintPathRole::SupportMaterialInterface
            | PrintPathRole::Ironing
                if is_first_layer && self.initial_layer_line_width > 0.0 =>
            {
                self.initial_layer_line_width
            }
            _ => self.width_for_role(role),
        }
    }

    pub fn extrusion_per_mm(
        &self,
        role: PrintPathRole,
        layer_height: f64,
    ) -> Result<f64, SliceError> {
        self.extrusion_per_mm_for_layer(role, layer_height, false)
    }

    pub fn extrusion_per_mm_for_layer(
        &self,
        role: PrintPathRole,
        layer_height: f64,
        is_first_layer: bool,
    ) -> Result<f64, SliceError> {
        self.extrusion_per_mm_for_layer_with_width(
            role,
            layer_height,
            is_first_layer,
            self.width_for_role_and_layer(role, is_first_layer),
        )
    }

    pub(crate) fn extrusion_per_mm_for_layer_with_width(
        &self,
        role: PrintPathRole,
        layer_height: f64,
        is_first_layer: bool,
        width: f64,
    ) -> Result<f64, SliceError> {
        if !layer_height.is_finite() || layer_height <= 0.0 {
            return Err(SliceError::InvalidInput(
                "layer height must be positive".to_owned(),
            ));
        }
        let thick_bridge_area = (role == PrintPathRole::Bridge && self.thick_bridges)
            || (role == PrintPathRole::InternalBridge && self.thick_internal_bridges);
        let mm3_per_mm = if thick_bridge_area {
            self.nozzle_diameter_for_role(role).powi(2)
                * self.bridge_flow
                * 0.25
                * std::f64::consts::PI
        } else {
            layer_height * (width - layer_height * (1.0 - std::f64::consts::PI / 4.0))
        };
        if mm3_per_mm <= 0.0 {
            return Err(SliceError::InvalidInput(
                "extrusion area must be positive".to_owned(),
            ));
        }
        let filament_area =
            std::f64::consts::PI * (self.filament_diameter_for_role(role) / 2.0).powi(2);
        let flow = match role {
            PrintPathRole::Bridge if self.thick_bridges => 1.0,
            PrintPathRole::Bridge => self.bridge_flow,
            PrintPathRole::InternalBridge => self.internal_bridge_flow,
            PrintPathRole::Brim => self.brim_flow_ratio,
            PrintPathRole::GapFill => self.gap_fill_flow_ratio,
            PrintPathRole::SparseInfill => self.sparse_infill_flow_ratio,
            PrintPathRole::SolidInfill => self.internal_solid_infill_flow_ratio,
            PrintPathRole::SupportMaterial => self.support_flow_ratio,
            PrintPathRole::SupportMaterialInterface => self.support_interface_flow_ratio,
            PrintPathRole::TopSolidInfill => self.top_solid_infill_flow_ratio,
            PrintPathRole::Ironing => self.ironing_flow_ratio,
            PrintPathRole::BottomSurface => self.bottom_solid_infill_flow_ratio,
            PrintPathRole::ExternalPerimeter => self.outer_wall_flow_ratio,
            PrintPathRole::OverhangPerimeter => self.overhang_flow_ratio,
            PrintPathRole::InternalPerimeter => self.inner_wall_flow_ratio,
            _ => 1.0,
        };
        let first_layer_flow = match role {
            PrintPathRole::ExternalPerimeter
            | PrintPathRole::InternalPerimeter
            | PrintPathRole::GapFill
            | PrintPathRole::SparseInfill
            | PrintPathRole::SolidInfill
            | PrintPathRole::TopSolidInfill
            | PrintPathRole::BottomSurface
            | PrintPathRole::SupportMaterial
            | PrintPathRole::SupportMaterialInterface
            | PrintPathRole::Ironing
                if is_first_layer =>
            {
                self.first_layer_flow_ratio
            }
            _ => 1.0,
        };
        Ok(mm3_per_mm / filament_area
            * self.filament_flow_ratio
            * self.print_flow_ratio
            * flow
            * first_layer_flow)
    }

    pub fn small_area_flow_multiplier_for_segment(
        &self,
        role: PrintPathRole,
        is_first_layer: bool,
        line_length_mm: f64,
    ) -> f64 {
        self.small_area_infill_flow_compensation
            .multiplier(role, is_first_layer, line_length_mm)
    }

    pub fn extrusion_delta_for_segment(
        &self,
        role: PrintPathRole,
        layer_height: f64,
        is_first_layer: bool,
        line_length_mm: f64,
    ) -> Result<f64, SliceError> {
        self.extrusion_delta_for_segment_with_width(ExplicitExtrusionSegment {
            role,
            layer_height,
            is_first_layer,
            line_width: self.width_for_role_and_layer(role, is_first_layer),
            line_length_mm,
        })
    }

    pub(crate) fn extrusion_delta_for_segment_with_width(
        &self,
        segment: ExplicitExtrusionSegment,
    ) -> Result<f64, SliceError> {
        Ok(segment.line_length_mm
            * self.extrusion_per_mm_for_layer_with_width(
                segment.role,
                segment.layer_height,
                segment.is_first_layer,
                segment.line_width,
            )?
            * self.small_area_flow_multiplier_for_segment(
                segment.role,
                segment.is_first_layer,
                segment.line_length_mm,
            ))
    }

    fn hardware_for_role(&self, role: PrintPathRole) -> RoleHardwareValues {
        match role {
            PrintPathRole::ExternalPerimeter
            | PrintPathRole::OverhangPerimeter
            | PrintPathRole::InternalPerimeter => self.hardware.wall,
            PrintPathRole::SparseInfill => self.hardware.sparse_infill,
            PrintPathRole::SolidInfill
            | PrintPathRole::TopSolidInfill
            | PrintPathRole::BottomSurface
            | PrintPathRole::Ironing => self.hardware.solid_infill,
            PrintPathRole::SupportMaterial => self.hardware.support,
            PrintPathRole::SupportMaterialInterface => self.hardware.support_interface,
            PrintPathRole::Skirt
            | PrintPathRole::Brim
            | PrintPathRole::GapFill
            | PrintPathRole::Bridge
            | PrintPathRole::InternalBridge => self.hardware.default,
        }
    }

    fn extrusion_hardware_for_role(&self, role: PrintPathRole) -> RoleHardwareValues {
        match role {
            PrintPathRole::SupportMaterial => self
                .support_material_extrusion_hardware
                .unwrap_or(self.hardware.support),
            _ => self.hardware_for_role(role),
        }
    }

    fn nozzle_diameter_for_role(&self, role: PrintPathRole) -> f64 {
        self.hardware_for_role(role).nozzle_diameter
    }

    fn filament_diameter_for_role(&self, role: PrintPathRole) -> f64 {
        self.extrusion_hardware_for_role(role).filament_diameter
    }
}
