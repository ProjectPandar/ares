#[cfg(test)]
mod tests;

use super::MotionOptions;
use crate::ExtrusionRole;
use crate::project_slice::perimeters::classic::materialize::FittedMove;

/// Feature string → the upstream `ExtrusionRole` discriminant used by the
/// `;_EXTRUSION_ROLE:n` markers (`GCode.cpp:6774`,
/// `ExtrusionEntity.hpp` enumerator order).
pub(super) fn role_id(feature: &str) -> u32 {
    match feature {
        "Inner wall" => 1,
        "Outer wall" => 2,
        "Overhang wall" => 3,
        "Sparse infill" => 4,
        "Solid infill" => 5,
        "Internal solid infill" => 5,
        "Top surface" => 6,
        "Bottom surface" => 7,
        "Ironing" => 8,
        "Bridge" => 9,
        "Internal bridge" | "Internal Bridge" => 10,
        "Gap fill" | "Gap infill" => 11,
        "Skirt" => 12,
        "Brim" => 13,
        "Support" => 14,
        "Support interface" => 15,
        "Support transition" => 16,
        "Prime tower" => 17,
        _ => 0,
    }
}

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
    /// The path belongs to a loop entity whose full closed point list
    /// was already handed to `CurlTracker::collect`
    /// (`ExtrusionLoop::collect_points` covers the complete closed loop,
    /// `ExtrusionEntity.hpp:507-512`); a path-level collect would
    /// duplicate the segments and, worse, score curvatures on the
    /// open fragment instead of the closed loop.
    pub(super) inside_loop: bool,
}

impl PathProperties<'_> {
    pub(super) fn kinematics(
        self,
        options: &MotionOptions,
        layer_index: usize,
        object_layer_over_raft: bool,
        path_length: f64,
    ) -> (u32, f64) {
        let speed = self.speed(options, layer_index, object_layer_over_raft, path_length);
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

    fn speed(
        &self,
        options: &MotionOptions,
        layer_index: usize,
        object_layer_over_raft: bool,
        path_length: f64,
    ) -> f64 {
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
        // `GCode.cpp:6564-6570`: the first-layer speed override applies to
        // `on_first_layer()` OR `object_layer_over_raft()` — the first
        // object layer over a raft prints at `initial_layer_speed` even
        // though its `layer->id()` is `raft_count`, not 0. Bottom surface
        // keeps its own role speed (`initial_layer_infill_speed` already).
        let first_layer_like = layer_index == 0 || object_layer_over_raft;
        let speed = if first_layer_like && self.feature != "Bottom surface" {
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

    /// `GCode.cpp:6618-6651`: on external perimeters, a volumetric-capped
    /// speed below `max_resonance_avoidance_speed` drops to the band `min`
    /// in the lower half and boosts to `max` in the upper half. The
    /// `m_resonance_avoidance` flag resets to true after every path, so it
    /// only gates the `speed > max` case the `speed < max` test already
    /// excludes.
    pub(super) fn resonance_avoided(self, options: &MotionOptions, speed: f64) -> f64 {
        let Some((min, max)) = options.resonance_avoidance else {
            return speed;
        };
        if self.feature != "Outer wall" || speed >= max {
            speed
        } else if speed < min + ((max - min) / 2.0) {
            speed.min(min)
        } else {
            max
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
            // `GCode.cpp:6548-6550`: interface paths use
            // `support_interface_speed`, not the support speed.
            "Support interface" => options.support_interface_speed,
            // `GCode.cpp:6540`: `erBottomSurface` always resolves to
            // `initial_layer_infill_speed` (and the first-layer override
            // at `:6568` deliberately skips it).
            "Bottom surface" => options.initial_layer_infill_speed,
            // `GCode.cpp:6524`: `erSupportTransition` shares the
            // bridge speed with overhang walls/bridge infill.
            "Support transition" => options.bridge_speed,
            // `GCode.cpp:4438-4441`: skirt entities are extruded with
            // an explicit `support_speed` argument (not the role
            // switch); `skirt_speed > 0` overrides it at `:6599`.
            "Skirt" => options.support_speed,
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

/// The extrude descriptions upstream passes per entity family
/// (`GCode.cpp:6144/6160/4438/4503`); only emitted under `gcode_comments`.
pub(super) fn feature_description(feature: &str) -> &'static str {
    match feature {
        "Skirt" => "skirt",
        "Brim" => "brim",
        "Inner wall" | "Outer wall" | "Overhang wall" => "perimeter",
        "Ironing" => "ironing",
        "Support" => "support material",
        "Support interface" => "support material interface",
        _ => "infill",
    }
}

/// `GCode::_extrude` appends " (bridge)" to the extrude description
/// whenever `is_bridge(role)` (`ExtrusionEntity.hpp:99-103`:
/// BridgeInfill | InternalBridgeInfill | OverhangPerimeter).
pub(super) fn is_bridge_feature(feature: &str) -> bool {
    matches!(feature, "Bridge" | "Internal Bridge" | "Overhang wall")
}
