use super::SpeedOptions;
use crate::{PrintPathRole, ToolpathMoveKind};

impl SpeedOptions {
    pub const fn travel_speed_mm_s(&self) -> f64 {
        self.travel_speed_mm_s
    }

    pub const fn travel_speed_z_mm_s(&self) -> f64 {
        self.travel_speed_z_mm_s
    }

    pub const fn external_perimeter_speed_mm_s(&self) -> f64 {
        self.external_perimeter_speed_mm_s
    }

    pub const fn sparse_infill_speed_mm_s(&self) -> f64 {
        self.sparse_infill_speed_mm_s
    }

    pub const fn skirt_speed_mm_s(&self) -> f64 {
        self.skirt_speed_mm_s
    }

    pub const fn bridge_speed_mm_s(&self) -> f64 {
        self.bridge_speed_mm_s
    }

    pub const fn small_perimeter_threshold_mm(&self) -> f64 {
        self.small_perimeter_threshold_mm
    }

    pub const fn small_perimeter_speed_mm_s(&self) -> f64 {
        self.small_perimeter_speed_mm_s
    }

    pub const fn filament_diameter_mm(&self) -> f64 {
        self.filament_diameter_mm
    }

    pub const fn filament_max_volumetric_speed_mm3_s(&self) -> f64 {
        self.filament_max_volumetric_speed_mm3_s
    }

    pub const fn resonance_avoidance(&self) -> bool {
        self.resonance_avoidance
    }

    pub const fn min_resonance_avoidance_speed_mm_s(&self) -> f64 {
        self.min_resonance_avoidance_speed_mm_s
    }

    pub const fn max_resonance_avoidance_speed_mm_s(&self) -> f64 {
        self.max_resonance_avoidance_speed_mm_s
    }

    pub const fn filament_adaptive_volumetric_speed(&self) -> bool {
        self.filament_adaptive_volumetric_speed
    }

    pub const fn volumetric_speed_coefficients(&self) -> Option<[f64; 6]> {
        self.volumetric_speed_coefficients
    }

    pub const fn max_volumetric_extrusion_rate_slope_mm3_s2(&self) -> f64 {
        self.max_volumetric_extrusion_rate_slope_mm3_s2
    }

    pub const fn max_volumetric_extrusion_rate_slope_segment_length_mm(&self) -> f64 {
        self.max_volumetric_extrusion_rate_slope_segment_length_mm
    }

    pub const fn extrusion_rate_smoothing_external_perimeter_only(&self) -> bool {
        self.extrusion_rate_smoothing_external_perimeter_only
    }

    pub const fn slow_down_layers(&self) -> u32 {
        self.slow_down_layers
    }

    pub const fn dont_slow_down_outer_wall(&self) -> bool {
        self.dont_slow_down_outer_wall
    }

    pub const fn slow_down_for_layer_cooling(&self) -> bool {
        self.slow_down_for_layer_cooling
    }

    pub const fn slow_down_layer_time_s(&self) -> f64 {
        self.slow_down_layer_time_s
    }

    pub const fn slow_down_min_speed_mm_s(&self) -> f64 {
        self.slow_down_min_speed_mm_s
    }

    pub const fn first_layer_speed_mm_s(&self) -> f64 {
        self.first_layer_speed_mm_s
    }

    pub const fn first_layer_infill_speed_mm_s(&self) -> f64 {
        self.first_layer_infill_speed_mm_s
    }

    pub fn overhang_speed_for_unsupported_span_mm(&self, unsupported_span_mm: f64) -> Option<f64> {
        self.overhang_speed_bands
            .speed_for_unsupported_span_mm(unsupported_span_mm)
    }

    pub const fn speed_for_role(&self, kind: ToolpathMoveKind, role: PrintPathRole) -> f64 {
        match kind {
            ToolpathMoveKind::Travel => self.travel_speed_mm_s,
            ToolpathMoveKind::Print => match role {
                PrintPathRole::Skirt => self.skirt_speed_mm_s,
                PrintPathRole::Brim => self.external_perimeter_speed_mm_s,
                PrintPathRole::Bridge => self.bridge_speed_mm_s,
                PrintPathRole::OverhangPerimeter => self.overhang_perimeter_speed_mm_s,
                PrintPathRole::InternalBridge => self.internal_bridge_speed_mm_s,
                PrintPathRole::GapFill => self.gap_infill_speed_mm_s,
                PrintPathRole::ExternalPerimeter => self.external_perimeter_speed_mm_s,
                PrintPathRole::InternalPerimeter => self.internal_perimeter_speed_mm_s,
                PrintPathRole::SparseInfill => self.sparse_infill_speed_mm_s,
                PrintPathRole::SolidInfill => self.internal_solid_infill_speed_mm_s,
                PrintPathRole::TopSolidInfill => self.top_surface_speed_mm_s,
                PrintPathRole::BottomSurface => self.first_layer_infill_speed_mm_s,
                PrintPathRole::SupportMaterial => self.support_speed_mm_s,
                PrintPathRole::SupportMaterialInterface => self.support_interface_speed_mm_s,
                PrintPathRole::Ironing => self.ironing_speed_mm_s,
            },
        }
    }

    pub const fn speed_for_layer(
        &self,
        kind: ToolpathMoveKind,
        role: PrintPathRole,
        is_first_layer: bool,
    ) -> f64 {
        match (is_first_layer, kind, role) {
            (true, ToolpathMoveKind::Travel, _) => self.first_layer_travel_speed_mm_s,
            (
                true,
                ToolpathMoveKind::Print,
                PrintPathRole::ExternalPerimeter
                | PrintPathRole::OverhangPerimeter
                | PrintPathRole::InternalPerimeter
                | PrintPathRole::Brim,
            ) => self.first_layer_speed_mm_s,
            (
                true,
                ToolpathMoveKind::Print,
                PrintPathRole::SparseInfill
                | PrintPathRole::SolidInfill
                | PrintPathRole::TopSolidInfill
                | PrintPathRole::BottomSurface
                | PrintPathRole::SupportMaterial
                | PrintPathRole::SupportMaterialInterface
                | PrintPathRole::Ironing,
            ) => self.first_layer_infill_speed_mm_s,
            _ => self.speed_for_role(kind, role),
        }
    }

    pub const fn feedrate_for_role(&self, kind: ToolpathMoveKind, role: PrintPathRole) -> f64 {
        self.speed_for_role(kind, role) * 60.0
    }

    pub const fn feedrate_for_layer(
        &self,
        kind: ToolpathMoveKind,
        role: PrintPathRole,
        is_first_layer: bool,
    ) -> f64 {
        self.speed_for_layer(kind, role, is_first_layer) * 60.0
    }

    pub const fn z_travel_speed_for_layer(&self, is_first_layer: bool) -> f64 {
        if self.travel_speed_z_mm_s == 0.0 {
            if is_first_layer {
                self.first_layer_travel_speed_mm_s
            } else {
                self.travel_speed_mm_s
            }
        } else {
            self.travel_speed_z_mm_s
        }
    }

    pub const fn z_travel_feedrate_for_layer(&self, is_first_layer: bool) -> f64 {
        self.z_travel_speed_for_layer(is_first_layer) * 60.0
    }

    pub const fn acceleration_for_layer(
        &self,
        kind: ToolpathMoveKind,
        role: PrintPathRole,
        is_first_layer: bool,
    ) -> Option<f64> {
        self.acceleration_options
            .acceleration_for_layer(kind, role, is_first_layer)
    }

    pub const fn jerk_for_layer(
        &self,
        kind: ToolpathMoveKind,
        role: PrintPathRole,
        is_first_layer: bool,
    ) -> Option<f64> {
        self.jerk_options.jerk_for_layer(kind, role, is_first_layer)
    }
}
