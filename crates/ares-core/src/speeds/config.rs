mod accessors;
mod overhang;

use crate::{AccelerationOptions, JerkOptions};

pub use overhang::OverhangSpeedBands;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SpeedOptions {
    travel_speed_mm_s: f64,
    first_layer_travel_speed_mm_s: f64,
    travel_speed_z_mm_s: f64,
    external_perimeter_speed_mm_s: f64,
    first_layer_speed_mm_s: f64,
    internal_perimeter_speed_mm_s: f64,
    sparse_infill_speed_mm_s: f64,
    internal_solid_infill_speed_mm_s: f64,
    support_speed_mm_s: f64,
    support_interface_speed_mm_s: f64,
    top_surface_speed_mm_s: f64,
    ironing_speed_mm_s: f64,
    first_layer_infill_speed_mm_s: f64,
    skirt_speed_mm_s: f64,
    bridge_speed_mm_s: f64,
    overhang_perimeter_speed_mm_s: f64,
    overhang_speed_bands: OverhangSpeedBands,
    internal_bridge_speed_mm_s: f64,
    gap_infill_speed_mm_s: f64,
    small_perimeter_threshold_mm: f64,
    small_perimeter_speed_mm_s: f64,
    filament_diameter_mm: f64,
    filament_max_volumetric_speed_mm3_s: f64,
    resonance_avoidance: bool,
    min_resonance_avoidance_speed_mm_s: f64,
    max_resonance_avoidance_speed_mm_s: f64,
    filament_adaptive_volumetric_speed: bool,
    volumetric_speed_coefficients: Option<[f64; 6]>,
    max_volumetric_extrusion_rate_slope_mm3_s2: f64,
    max_volumetric_extrusion_rate_slope_segment_length_mm: f64,
    extrusion_rate_smoothing_external_perimeter_only: bool,
    slow_down_layers: u32,
    dont_slow_down_outer_wall: bool,
    slow_down_for_layer_cooling: bool,
    slow_down_layer_time_s: f64,
    slow_down_min_speed_mm_s: f64,
    acceleration_options: AccelerationOptions,
    jerk_options: JerkOptions,
}

impl SpeedOptions {
    pub const fn new(
        travel_speed_mm_s: f64,
        external_perimeter_speed_mm_s: f64,
        sparse_infill_speed_mm_s: f64,
    ) -> Self {
        Self {
            travel_speed_mm_s,
            first_layer_travel_speed_mm_s: travel_speed_mm_s,
            travel_speed_z_mm_s: 0.0,
            external_perimeter_speed_mm_s,
            first_layer_speed_mm_s: 30.0,
            internal_perimeter_speed_mm_s: external_perimeter_speed_mm_s,
            sparse_infill_speed_mm_s,
            internal_solid_infill_speed_mm_s: sparse_infill_speed_mm_s,
            support_speed_mm_s: 80.0,
            support_interface_speed_mm_s: 80.0,
            top_surface_speed_mm_s: 100.0,
            ironing_speed_mm_s: 20.0,
            first_layer_infill_speed_mm_s: 60.0,
            skirt_speed_mm_s: external_perimeter_speed_mm_s,
            bridge_speed_mm_s: external_perimeter_speed_mm_s,
            overhang_perimeter_speed_mm_s: external_perimeter_speed_mm_s,
            overhang_speed_bands: OverhangSpeedBands::disabled(0.0),
            internal_bridge_speed_mm_s: external_perimeter_speed_mm_s,
            gap_infill_speed_mm_s: 30.0,
            small_perimeter_threshold_mm: 0.0,
            small_perimeter_speed_mm_s: external_perimeter_speed_mm_s * 0.5,
            filament_diameter_mm: 1.75,
            filament_max_volumetric_speed_mm3_s: 0.0,
            resonance_avoidance: false,
            min_resonance_avoidance_speed_mm_s: 70.0,
            max_resonance_avoidance_speed_mm_s: 120.0,
            filament_adaptive_volumetric_speed: false,
            volumetric_speed_coefficients: None,
            max_volumetric_extrusion_rate_slope_mm3_s2: 0.0,
            max_volumetric_extrusion_rate_slope_segment_length_mm: 3.0,
            extrusion_rate_smoothing_external_perimeter_only: false,
            slow_down_layers: 0,
            dont_slow_down_outer_wall: false,
            slow_down_for_layer_cooling: false,
            slow_down_layer_time_s: 5.0,
            slow_down_min_speed_mm_s: 10.0,
            acceleration_options: AccelerationOptions {
                default_mm_s2: 500.0,
                initial_layer_mm_s2: 300.0,
                outer_wall_mm_s2: 500.0,
                bridge_mm_s2: 250.0,
                inner_wall_mm_s2: 10000.0,
                travel_mm_s2: 10000.0,
                initial_layer_travel_mm_s2: 10000.0,
                sparse_infill_mm_s2: 500.0,
                internal_solid_infill_mm_s2: 500.0,
                top_surface_mm_s2: 500.0,
            },
            jerk_options: JerkOptions {
                default_mm_s: 0.0,
                initial_layer_mm_s: 9.0,
                outer_wall_mm_s: 9.0,
                inner_wall_mm_s: 9.0,
                infill_mm_s: 9.0,
                top_surface_mm_s: 9.0,
                travel_mm_s: 12.0,
                initial_layer_travel_mm_s: 12.0,
            },
        }
    }

    pub const fn with_skirt_speed(self, skirt_speed_mm_s: f64) -> Self {
        Self {
            skirt_speed_mm_s,
            ..self
        }
    }

    pub const fn with_first_layer_speed(self, first_layer_speed_mm_s: f64) -> Self {
        Self {
            first_layer_speed_mm_s,
            ..self
        }
    }

    pub const fn with_first_layer_infill_speed(self, first_layer_infill_speed_mm_s: f64) -> Self {
        Self {
            first_layer_infill_speed_mm_s,
            ..self
        }
    }

    pub const fn with_first_layer_travel_speed(self, first_layer_travel_speed_mm_s: f64) -> Self {
        Self {
            first_layer_travel_speed_mm_s,
            ..self
        }
    }

    pub const fn with_travel_speed_z(self, travel_speed_z_mm_s: f64) -> Self {
        Self {
            travel_speed_z_mm_s,
            ..self
        }
    }

    pub const fn with_bridge_speed(self, bridge_speed_mm_s: f64) -> Self {
        Self {
            bridge_speed_mm_s,
            overhang_perimeter_speed_mm_s: bridge_speed_mm_s,
            ..self
        }
    }

    pub const fn with_overhang_perimeter_speed(self, overhang_perimeter_speed_mm_s: f64) -> Self {
        Self {
            overhang_perimeter_speed_mm_s,
            ..self
        }
    }

    pub const fn with_overhang_speed_bands(self, overhang_speed_bands: OverhangSpeedBands) -> Self {
        Self {
            overhang_speed_bands,
            ..self
        }
    }

    pub const fn with_internal_bridge_speed(self, internal_bridge_speed_mm_s: f64) -> Self {
        Self {
            internal_bridge_speed_mm_s,
            ..self
        }
    }

    pub const fn with_gap_infill_speed(self, gap_infill_speed_mm_s: f64) -> Self {
        Self {
            gap_infill_speed_mm_s,
            ..self
        }
    }

    pub const fn with_internal_perimeter_speed(self, internal_perimeter_speed_mm_s: f64) -> Self {
        Self {
            internal_perimeter_speed_mm_s,
            ..self
        }
    }

    pub const fn with_internal_solid_infill_speed(
        self,
        internal_solid_infill_speed_mm_s: f64,
    ) -> Self {
        Self {
            internal_solid_infill_speed_mm_s,
            ..self
        }
    }

    pub const fn with_support_speed(self, support_speed_mm_s: f64) -> Self {
        Self {
            support_speed_mm_s,
            ..self
        }
    }

    pub const fn with_support_interface_speed(self, support_interface_speed_mm_s: f64) -> Self {
        Self {
            support_interface_speed_mm_s,
            ..self
        }
    }

    pub const fn with_top_surface_speed(self, top_surface_speed_mm_s: f64) -> Self {
        Self {
            top_surface_speed_mm_s,
            ..self
        }
    }

    pub const fn with_ironing_speed(self, ironing_speed_mm_s: f64) -> Self {
        Self {
            ironing_speed_mm_s,
            ..self
        }
    }

    pub const fn with_small_perimeter_threshold(self, small_perimeter_threshold_mm: f64) -> Self {
        Self {
            small_perimeter_threshold_mm,
            ..self
        }
    }

    pub const fn with_small_perimeter_speed(self, small_perimeter_speed_mm_s: f64) -> Self {
        Self {
            small_perimeter_speed_mm_s,
            ..self
        }
    }

    pub const fn with_filament_diameter(self, filament_diameter_mm: f64) -> Self {
        Self {
            filament_diameter_mm,
            ..self
        }
    }

    pub const fn with_filament_max_volumetric_speed(
        self,
        filament_max_volumetric_speed_mm3_s: f64,
    ) -> Self {
        Self {
            filament_max_volumetric_speed_mm3_s,
            ..self
        }
    }

    pub const fn with_resonance_avoidance(
        self,
        resonance_avoidance: bool,
        min_resonance_avoidance_speed_mm_s: f64,
        max_resonance_avoidance_speed_mm_s: f64,
    ) -> Self {
        Self {
            resonance_avoidance,
            min_resonance_avoidance_speed_mm_s,
            max_resonance_avoidance_speed_mm_s,
            ..self
        }
    }

    pub const fn with_filament_adaptive_volumetric_speed(
        self,
        filament_adaptive_volumetric_speed: bool,
    ) -> Self {
        Self {
            filament_adaptive_volumetric_speed,
            ..self
        }
    }

    pub const fn with_volumetric_speed_coefficients(
        self,
        volumetric_speed_coefficients: Option<[f64; 6]>,
    ) -> Self {
        Self {
            volumetric_speed_coefficients,
            ..self
        }
    }

    pub const fn with_max_volumetric_extrusion_rate_slope(
        self,
        max_volumetric_extrusion_rate_slope_mm3_s2: f64,
    ) -> Self {
        Self {
            max_volumetric_extrusion_rate_slope_mm3_s2,
            ..self
        }
    }

    pub const fn with_max_volumetric_extrusion_rate_slope_segment_length(
        self,
        max_volumetric_extrusion_rate_slope_segment_length_mm: f64,
    ) -> Self {
        Self {
            max_volumetric_extrusion_rate_slope_segment_length_mm,
            ..self
        }
    }

    pub const fn with_extrusion_rate_smoothing_external_perimeter_only(
        self,
        extrusion_rate_smoothing_external_perimeter_only: bool,
    ) -> Self {
        Self {
            extrusion_rate_smoothing_external_perimeter_only,
            ..self
        }
    }

    pub const fn with_slow_down_layers(self, slow_down_layers: u32) -> Self {
        Self {
            slow_down_layers,
            ..self
        }
    }

    pub const fn with_dont_slow_down_outer_wall(self, dont_slow_down_outer_wall: bool) -> Self {
        Self {
            dont_slow_down_outer_wall,
            ..self
        }
    }

    pub const fn with_slow_down_for_layer_cooling(self, slow_down_for_layer_cooling: bool) -> Self {
        Self {
            slow_down_for_layer_cooling,
            ..self
        }
    }

    pub const fn with_slow_down_layer_time(self, slow_down_layer_time_s: f64) -> Self {
        Self {
            slow_down_layer_time_s,
            ..self
        }
    }

    pub const fn with_slow_down_min_speed(self, slow_down_min_speed_mm_s: f64) -> Self {
        Self {
            slow_down_min_speed_mm_s,
            ..self
        }
    }

    pub const fn with_acceleration_options(
        self,
        acceleration_options: AccelerationOptions,
    ) -> Self {
        Self {
            acceleration_options,
            ..self
        }
    }

    pub const fn with_jerk_options(self, jerk_options: JerkOptions) -> Self {
        Self {
            jerk_options,
            ..self
        }
    }
}
