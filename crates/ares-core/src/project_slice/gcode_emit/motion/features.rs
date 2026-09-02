#[cfg(test)]
mod tests;

use super::MotionOptions;
use crate::ExtrusionRole;
use crate::project_slice::perimeters::classic::materialize::FittedMove;

#[derive(Clone, Copy)]
pub(super) struct PathProperties<'a> {
    pub(super) mm3_per_mm: f64,
    pub(super) width: f32,
    pub(super) height: f32,
    pub(super) feature: &'static str,
    pub(super) is_perimeter: bool,
    pub(super) end_clip: f64,
    pub(super) fitting: &'a [FittedMove],
    pub(super) slope: Option<super::scarf::Slope>,
}

impl PathProperties<'_> {
    pub(super) fn kinematics(
        self,
        options: &MotionOptions,
        layer_index: usize,
        path_length: f64,
    ) -> (u32, f64) {
        let speed = self.speed(options, layer_index, path_length);
        // `GCode.cpp:6414`: default acceleration of zero disables all per-print
        // acceleration switches.
        if options.default_acceleration == 0 {
            return (0, speed);
        }
        // `GCode.cpp:6418-6438`: first layer uses its own value when set,
        // otherwise each role picks only an option above zero and every
        // remaining case falls back to the default.
        let acceleration = if layer_index == 0 && options.initial_layer_acceleration > 0 {
            options.initial_layer_acceleration
        } else {
            match self.feature {
                "Bridge" | "Overhang wall" | "Internal Bridge"
                    if options.bridge_acceleration > 0 =>
                {
                    options.bridge_acceleration
                }
                "Sparse infill" if options.sparse_infill_acceleration > 0 => {
                    options.sparse_infill_acceleration
                }
                "Internal solid infill" if options.internal_solid_infill_acceleration > 0 => {
                    options.internal_solid_infill_acceleration
                }
                "Outer wall" if options.outer_wall_acceleration > 0 => {
                    options.outer_wall_acceleration
                }
                "Inner wall" if options.inner_wall_acceleration > 0 => {
                    options.inner_wall_acceleration
                }
                "Top surface" if options.top_surface_acceleration > 0 => {
                    options.top_surface_acceleration
                }
                _ => options.default_acceleration,
            }
        };
        (acceleration, speed)
    }
}

impl PathProperties<'_> {
    pub(super) fn jerk(self, options: &MotionOptions, layer_index: usize) -> f64 {
        if options.default_jerk <= 0.0 {
            return 0.0;
        }
        if layer_index == 0 && options.initial_layer_jerk > 0.0 {
            return options.initial_layer_jerk;
        }
        match self.feature {
            "Outer wall" | "Overhang wall" if options.outer_wall_jerk > 0.0 => {
                options.outer_wall_jerk
            }
            "Inner wall" if options.inner_wall_jerk > 0.0 => options.inner_wall_jerk,
            "Top surface" if options.top_surface_jerk > 0.0 => options.top_surface_jerk,
            "Sparse infill"
            | "Internal solid infill"
            | "Bottom surface"
            | "Bridge"
            | "Internal Bridge"
            | "Gap infill"
                if options.infill_jerk > 0.0 =>
            {
                options.infill_jerk
            }
            _ => options.default_jerk,
        }
    }

    fn speed(&self, options: &MotionOptions, layer_index: usize, path_length: f64) -> f64 {
        if let Some(slope) = self.slope {
            return slope.speed;
        }
        let is_perimeter = matches!(self.feature, "Inner wall" | "Outer wall" | "Overhang wall");
        // The small-perimeter pre-selection replaces the role speed before
        // the `_extrude` chain runs (`GCode.cpp:5809-5813`).
        let base = if layer_index > 0
            && matches!(self.feature, "Inner wall" | "Outer wall")
            && path_length <= options.small_perimeter_threshold * 2.0 * std::f64::consts::PI
        {
            options.small_perimeter_speed
        } else {
            self.role_speed(options)
        };
        let speed = if layer_index == 0 {
            if is_perimeter {
                options.initial_layer_speed
            } else {
                options.initial_layer_infill_speed
            }
        } else {
            self.slow_down_layer_ramp(options, layer_index, is_perimeter, base)
        };
        // `GCode.cpp:6599-6604`: a positive skirt speed overrides the role
        // default on every layer; zero keeps the layer default.
        if self.feature == "Skirt" && options.skirt_speed > 0.0 {
            options.skirt_speed
        } else {
            speed
        }
    }

    /// `GCode.cpp:6569-6588`: with `slow_down_layers > 1`, early layers ramp
    /// from the first-layer speed toward the configured speed via
    /// `lerp(first, speed, layer/N)`; the raft variant anchors the ramp
    /// above the raft layers.
    fn slow_down_layer_ramp(
        &self,
        options: &MotionOptions,
        layer_index: usize,
        is_perimeter: bool,
        speed: f64,
    ) -> f64 {
        if options.slow_down_layers <= 1 {
            return speed;
        }
        let ramp_layers = options.slow_down_layers as usize;
        let offset = if options.raft_layers == 0 {
            if layer_index == 0 || layer_index >= ramp_layers {
                return speed;
            }
            layer_index
        } else {
            let raft_layers = options.raft_layers as usize;
            if layer_index <= raft_layers || layer_index - raft_layers >= ramp_layers {
                return speed;
            }
            layer_index - raft_layers
        };
        let first_layer_speed = if is_perimeter {
            options.initial_layer_speed
        } else {
            options.initial_layer_infill_speed
        };
        if first_layer_speed >= speed {
            return speed;
        }
        let factor = offset as f64 / ramp_layers as f64;
        speed.min(first_layer_speed + factor * (speed - first_layer_speed))
    }

    fn role_speed(&self, options: &MotionOptions) -> f64 {
        match self.feature {
            "Outer wall" => options.outer_wall_speed,
            "Bridge" | "Overhang wall" => options.bridge_speed,
            "Internal Bridge" => options.internal_bridge_speed,
            "Top surface" => options.top_surface_speed,
            "Ironing" => options.ironing_speed,
            "Sparse infill" => options.sparse_infill_speed,
            "Internal solid infill" => options.internal_solid_infill_speed,
            "Gap infill" => options.gap_infill_speed,
            "Brim" | "Support" => options.support_speed,
            _ => options.inner_wall_speed,
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
        ExtrusionRole::BridgeInfill => "Bridge",
        ExtrusionRole::InternalBridgeInfill => "Internal Bridge",
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
