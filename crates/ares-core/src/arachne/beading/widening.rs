use super::base::{Beading, BeadingStrategy, BeadingStrategyConfig};

pub(crate) struct WideningBeadingStrategy {
    config: BeadingStrategyConfig,
    parent: Box<dyn BeadingStrategy>,
    min_input_width: i64,
    min_output_width: i64,
}

impl WideningBeadingStrategy {
    pub(crate) fn new(
        parent: Box<dyn BeadingStrategy>,
        min_input_width: i64,
        min_output_width: i64,
    ) -> Self {
        Self {
            config: *parent.config(),
            parent,
            min_input_width,
            min_output_width,
        }
    }
}

impl BeadingStrategy for WideningBeadingStrategy {
    fn config(&self) -> &BeadingStrategyConfig {
        &self.config
    }

    fn compute(&self, thickness: i64, bead_count: i64) -> Beading {
        if bead_count <= 1 && thickness < self.transition_thickness(1) {
            let mut result = Beading::empty(thickness);
            if thickness >= self.min_input_width {
                result
                    .bead_widths
                    .push(thickness.max(self.min_output_width));
                result.toolpath_locations.push(thickness / 2);
                result.left_over = 0;
            }
            result
        } else {
            self.parent.compute(thickness, bead_count)
        }
    }

    fn optimal_thickness(&self, bead_count: i64) -> i64 {
        self.parent.optimal_thickness(bead_count)
    }

    fn transition_thickness(&self, lower_bead_count: i64) -> i64 {
        if lower_bead_count == 0 {
            self.min_input_width
        } else {
            self.parent.transition_thickness(lower_bead_count)
        }
    }

    fn optimal_bead_count(&self, thickness: i64) -> i64 {
        if thickness < self.min_input_width {
            return 0;
        }
        self.parent.optimal_bead_count(thickness).max(1)
    }

    fn transitioning_length(&self, lower_bead_count: i64) -> i64 {
        self.parent.transitioning_length(lower_bead_count)
    }

    fn transition_anchor_pos(&self, lower_bead_count: i64) -> f32 {
        self.parent.transition_anchor_pos(lower_bead_count)
    }

    fn nonlinear_thicknesses(&self, lower_bead_count: i64) -> Vec<i64> {
        let mut result = vec![self.min_output_width];
        result.extend(self.parent.nonlinear_thicknesses(lower_bead_count));
        result
    }

    fn description(&self) -> String {
        format!("Widening+{}", self.parent.description())
    }
}
