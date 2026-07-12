use super::{
    InfillOptions, InfillPattern, InfillWallBoundaryOptions, InfillWallOverlapOptions,
};
use crate::ShellLayerOptions;

impl InfillOptions {
    pub(crate) fn new_for_tests(density: f64, direction: f64, line_width: f64) -> Self {
        Self {
            sparse_density_percent: density,
            direction_degrees: direction,
            sparse_infill_rotate_template_degrees: Vec::new(),
            line_width,
            fill_multiline: 1,
            solid_line_width: line_width,
            minimum_sparse_infill_area_mm2: 15.0,
            pattern: InfillPattern::Rectilinear,
            solid_direction_degrees: direction,
            bridge_angle_degrees: 0.0,
            internal_bridge_angle_degrees: 0.0,
            bridge_density_percent: 100.0,
            internal_bridge_density_percent: 100.0,
            internal_bridge_filter: super::internal_bridge_filter::InternalBridgeFilter::Disabled,
            top_surface_density_percent: 100.0,
            min_width_top_surface_mm: 0.0,
            calib_flowrate_topinfill_special_order: false,
            bottom_surface_density_percent: 100.0,
            elephant_foot_layers_density_percent: 100.0,
            elephant_foot_compensation_layers: 1,
            solid_infill_rotate_template_degrees: Vec::new(),
            internal_solid_infill_pattern: InfillPattern::Monotonic,
            bottom_surface_pattern: InfillPattern::Monotonic,
            top_surface_pattern: InfillPattern::MonotonicLine,
            extra_solid_infills: super::extra_solid::ExtraSolidInfills::default(),
            detect_narrow_internal_solid_infill: true,
            shell_layers: ShellLayerOptions::new(0, 0),
            spiral_mode: false,
            symmetric_infill_y_axis: false,
            infill_combination: false,
            infill_combination_max_layer_height_mm: 0.4,
            infill_anchor_length_mm: 0.0,
            infill_shift_step_mm: 0.4,
            wall_overlap: InfillWallOverlapOptions::new_for_tests(15.0, 25.0),
            wall_boundary: InfillWallBoundaryOptions::new_for_tests(0, line_width, line_width),
        }
    }

    pub(crate) fn with_minimum_sparse_infill_area_for_tests(
        self,
        minimum_sparse_infill_area_mm2: f64,
    ) -> Self {
        Self {
            minimum_sparse_infill_area_mm2,
            ..self
        }
    }

    pub(crate) fn with_pattern_for_tests(self, pattern: InfillPattern) -> Self {
        Self { pattern, ..self }
    }

    pub(crate) fn with_symmetric_infill_y_axis_for_tests(
        self,
        symmetric_infill_y_axis: bool,
    ) -> Self {
        Self {
            symmetric_infill_y_axis,
            ..self
        }
    }

    pub(crate) fn with_infill_combination_for_tests(self, max_layer_height_mm: f64) -> Self {
        Self {
            infill_combination: true,
            infill_combination_max_layer_height_mm: max_layer_height_mm,
            ..self
        }
    }

    pub(crate) fn with_sparse_infill_rotate_template_for_tests(
        self,
        sparse_infill_rotate_template_degrees: Vec<f64>,
    ) -> Self {
        Self {
            sparse_infill_rotate_template_degrees,
            ..self
        }
    }

    pub(crate) fn with_infill_shift_step_for_tests(self, infill_shift_step_mm: f64) -> Self {
        Self {
            infill_shift_step_mm,
            ..self
        }
    }

    pub(crate) fn with_fill_multiline_for_tests(self, fill_multiline: usize) -> Self {
        Self {
            fill_multiline,
            ..self
        }
    }

    pub(crate) fn with_bridge_angle_for_tests(self, bridge_angle_degrees: f64) -> Self {
        Self {
            bridge_angle_degrees,
            ..self
        }
    }

    pub(crate) fn with_internal_bridge_angle_for_tests(
        self,
        internal_bridge_angle_degrees: f64,
    ) -> Self {
        Self {
            internal_bridge_angle_degrees,
            ..self
        }
    }

    pub(crate) fn with_bridge_density_for_tests(self, bridge_density_percent: f64) -> Self {
        Self {
            bridge_density_percent,
            ..self
        }
    }

    pub(crate) fn with_internal_bridge_density_for_tests(
        self,
        internal_bridge_density_percent: f64,
    ) -> Self {
        Self {
            internal_bridge_density_percent,
            ..self
        }
    }

    pub(crate) fn with_internal_bridge_filter_for_tests(self, value: &str) -> Self {
        Self {
            internal_bridge_filter: super::internal_bridge_filter::InternalBridgeFilter::parse(
                Some(&serde_json::Value::String(value.to_owned())),
            )
            .unwrap(),
            ..self
        }
    }

    pub(crate) const fn internal_bridge_filter_for_tests(&self) -> &'static str {
        self.internal_bridge_filter.as_str()
    }

    pub(crate) fn with_top_surface_density_for_tests(
        self,
        top_surface_density_percent: f64,
    ) -> Self {
        Self {
            top_surface_density_percent,
            ..self
        }
    }

    pub(crate) fn with_calib_flowrate_topinfill_special_order_for_tests(
        self,
        value: bool,
    ) -> Self {
        Self {
            calib_flowrate_topinfill_special_order: value,
            ..self
        }
    }

    pub(crate) fn with_bottom_surface_density_for_tests(
        self,
        bottom_surface_density_percent: f64,
    ) -> Self {
        Self {
            bottom_surface_density_percent,
            ..self
        }
    }

    pub(crate) fn with_elephant_foot_layers_density_for_tests(
        self,
        elephant_foot_layers_density_percent: f64,
    ) -> Self {
        Self {
            elephant_foot_layers_density_percent,
            ..self
        }
    }

    pub(crate) fn with_elephant_foot_compensation_layers_for_tests(
        self,
        elephant_foot_compensation_layers: usize,
    ) -> Self {
        Self {
            elephant_foot_compensation_layers,
            ..self
        }
    }

    pub(crate) fn with_internal_solid_infill_pattern_for_tests(
        self,
        internal_solid_infill_pattern: InfillPattern,
    ) -> Self {
        Self {
            internal_solid_infill_pattern,
            ..self
        }
    }

    pub(crate) fn with_bottom_surface_pattern_for_tests(
        self,
        bottom_surface_pattern: InfillPattern,
    ) -> Self {
        Self {
            bottom_surface_pattern,
            ..self
        }
    }

    pub(crate) fn with_top_surface_pattern_for_tests(
        self,
        top_surface_pattern: InfillPattern,
    ) -> Self {
        Self {
            top_surface_pattern,
            ..self
        }
    }

    pub(crate) fn extra_solid_infills_matches_layer_for_tests(&self, layer_index: usize) -> bool {
        self.extra_solid_infills.matches_layer(layer_index)
    }

    pub(crate) fn with_extra_solid_infills_for_tests(self, pattern: &str) -> Self {
        Self {
            extra_solid_infills: super::extra_solid::ExtraSolidInfills::parse(Some(
                &serde_json::Value::String(pattern.to_owned()),
            ))
            .unwrap(),
            ..self
        }
    }

    pub(crate) fn with_detect_narrow_internal_solid_infill_for_tests(
        self,
        detect_narrow_internal_solid_infill: bool,
    ) -> Self {
        Self {
            detect_narrow_internal_solid_infill,
            ..self
        }
    }

    pub(crate) fn with_shell_layers_for_tests(
        self,
        bottom_shell_layers: usize,
        top_shell_layers: usize,
    ) -> Self {
        Self {
            shell_layers: ShellLayerOptions::new(bottom_shell_layers, top_shell_layers),
            ..self
        }
    }

    pub(crate) fn with_spiral_mode_for_tests(self, spiral_mode: bool) -> Self {
        Self {
            spiral_mode,
            ..self
        }
    }

    pub(crate) fn with_solid_infill_rotate_template_for_tests(
        self,
        solid_infill_rotate_template_degrees: Vec<f64>,
    ) -> Self {
        Self {
            solid_infill_rotate_template_degrees,
            ..self
        }
    }

    pub(crate) fn with_wall_boundary_for_tests(
        self,
        wall_loops: u32,
        external_wall_line_width: f64,
        internal_wall_line_width: f64,
    ) -> Self {
        Self {
            wall_boundary: InfillWallBoundaryOptions::new_for_tests(
                wall_loops,
                external_wall_line_width,
                internal_wall_line_width,
            ),
            ..self
        }
    }

    pub(crate) fn with_solid_line_width_for_tests(self, solid_line_width: f64) -> Self {
        Self {
            solid_line_width,
            ..self
        }
    }

    pub(crate) fn with_infill_wall_overlap_for_tests(self, percent: f64) -> Self {
        Self {
            wall_overlap: InfillWallOverlapOptions::new_for_tests(
                percent,
                self.wall_overlap.top_bottom_percent(),
            ),
            ..self
        }
    }

    pub(crate) fn with_top_bottom_infill_wall_overlap_for_tests(self, percent: f64) -> Self {
        Self {
            wall_overlap: InfillWallOverlapOptions::new_for_tests(
                self.wall_overlap.infill_percent(),
                percent,
            ),
            ..self
        }
    }

    pub(crate) fn with_only_one_wall_first_layer_for_tests(self) -> Self {
        Self {
            wall_boundary: self
                .wall_boundary
                .with_only_one_wall_first_layer_for_tests(),
            ..self
        }
    }

    pub(crate) fn with_only_one_wall_top_for_tests(self) -> Self {
        Self {
            wall_boundary: self.wall_boundary.with_only_one_wall_top_for_tests(),
            ..self
        }
    }

    pub(crate) fn with_alternate_extra_wall_for_tests(self) -> Self {
        Self {
            wall_boundary: self.wall_boundary.with_alternate_extra_wall_for_tests(),
            ..self
        }
    }
}
