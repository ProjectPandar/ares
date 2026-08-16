use super::MotionOptions;
use crate::ExtrusionRole;

#[derive(Clone, Copy)]
pub(super) struct PathProperties {
    pub(super) mm3_per_mm: f64,
    pub(super) width: f32,
    pub(super) feature: &'static str,
    pub(super) is_perimeter: bool,
}

impl PathProperties {
    pub(super) fn kinematics(self, options: &MotionOptions, layer_index: usize) -> (u32, f64) {
        if layer_index == 0 {
            let speed = if self.feature == "Bottom surface" {
                options.initial_layer_infill_speed
            } else {
                options.initial_layer_speed
            };
            return (options.initial_layer_acceleration, speed);
        }
        match self.feature {
            "Outer wall" => (options.outer_wall_acceleration, options.outer_wall_speed),
            "Top surface" => (options.top_surface_acceleration, options.top_surface_speed),
            "Sparse infill" => (options.default_acceleration, options.sparse_infill_speed),
            "Internal solid infill" => (
                options.default_acceleration,
                options.internal_solid_infill_speed,
            ),
            "Gap infill" => (options.default_acceleration, options.gap_infill_speed),
            _ => (options.default_acceleration, options.inner_wall_speed),
        }
    }
}

pub(super) fn for_fill(role: ExtrusionRole) -> &'static str {
    match role {
        ExtrusionRole::InternalInfill => "Sparse infill",
        ExtrusionRole::SolidInfill => "Internal solid infill",
        ExtrusionRole::TopSolidInfill => "Top surface",
        ExtrusionRole::BottomSurface => "Bottom surface",
        ExtrusionRole::Ironing => "Ironing",
        ExtrusionRole::BridgeInfill | ExtrusionRole::InternalBridgeInfill => "Bridge",
        ExtrusionRole::GapFill => "Gap infill",
        ExtrusionRole::Skirt => "Skirt",
        ExtrusionRole::Brim => "Brim",
        ExtrusionRole::SupportMaterial => "Support",
        ExtrusionRole::SupportMaterialInterface => "Support interface",
        ExtrusionRole::SupportTransition => "Support transition",
        ExtrusionRole::WipeTower => "Prime tower",
        ExtrusionRole::Custom => "Custom",
        ExtrusionRole::Perimeter => "Inner wall",
        ExtrusionRole::ExternalPerimeter => "Outer wall",
        ExtrusionRole::OverhangPerimeter => "Overhang wall",
        ExtrusionRole::None | ExtrusionRole::Mixed => "Mixed",
    }
}
