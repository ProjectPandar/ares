use super::base::{Beading, BeadingStrategy, BeadingStrategyConfig};

pub(crate) struct RedistributeBeadingStrategy {
    config: BeadingStrategyConfig,
    parent: Box<dyn BeadingStrategy>,
    optimal_width_outer: i64,
    minimum_variable_line_ratio: f64,
}

impl RedistributeBeadingStrategy {
    pub(crate) fn new(
        optimal_width_outer: i64,
        minimum_variable_line_ratio: f64,
        parent: Box<dyn BeadingStrategy>,
    ) -> Self {
        Self {
            config: *parent.config(),
            parent,
            optimal_width_outer,
            minimum_variable_line_ratio,
        }
    }
}

impl BeadingStrategy for RedistributeBeadingStrategy {
    fn config(&self) -> &BeadingStrategyConfig {
        &self.config
    }

    fn compute(&self, thickness: i64, bead_count: i64) -> Beading {
        if bead_count == 0
            || (thickness as f64)
                < self.minimum_variable_line_ratio * self.optimal_width_outer as f64
        {
            return Beading::empty(thickness);
        }

        let inner_bead_count = bead_count - 2;
        let inner_thickness = thickness - 2 * self.optimal_width_outer;
        let mut result = if inner_bead_count > 0 && inner_thickness > 0 {
            let mut inner = self.parent.compute(inner_thickness, inner_bead_count);
            for location in &mut inner.toolpath_locations {
                *location += self.optimal_width_outer;
            }
            inner
        } else {
            Beading::empty(thickness)
        };

        let actual_outer_thickness = if bead_count > 2 {
            (thickness / 2).min(self.optimal_width_outer)
        } else {
            thickness / bead_count
        };
        result.bead_widths.insert(0, actual_outer_thickness);
        result
            .toolpath_locations
            .insert(0, actual_outer_thickness / 2);
        if bead_count > 1 {
            result.bead_widths.push(actual_outer_thickness);
            result
                .toolpath_locations
                .push(thickness - actual_outer_thickness / 2);
        }
        result.total_thickness = thickness;
        result.left_over = thickness - result.bead_widths.iter().sum::<i64>();
        result
    }

    fn optimal_thickness(&self, bead_count: i64) -> i64 {
        let inner_bead_count = 0.max(bead_count - 2);
        let outer_bead_count = bead_count - inner_bead_count;
        self.parent.optimal_thickness(inner_bead_count)
            + self.optimal_width_outer * outer_bead_count
    }

    fn transition_thickness(&self, lower_bead_count: i64) -> i64 {
        match lower_bead_count {
            0 => (self.minimum_variable_line_ratio * self.optimal_width_outer as f64) as i64,
            1 => {
                ((1.0 + self.parent.split_middle_threshold()) * self.optimal_width_outer as f64)
                    as i64
            }
            count => self.parent.transition_thickness(count - 2) + 2 * self.optimal_width_outer,
        }
    }

    fn optimal_bead_count(&self, thickness: i64) -> i64 {
        if (thickness as f64) < self.minimum_variable_line_ratio * self.optimal_width_outer as f64 {
            return 0;
        }
        if thickness <= 2 * self.optimal_width_outer {
            return if thickness as f64
                > (1.0 + self.parent.split_middle_threshold()) * self.optimal_width_outer as f64
            {
                2
            } else {
                1
            };
        }
        self.parent
            .optimal_bead_count(thickness - 2 * self.optimal_width_outer)
            + 2
    }

    fn transitioning_length(&self, lower_bead_count: i64) -> i64 {
        self.parent.transitioning_length(lower_bead_count)
    }

    fn transition_anchor_pos(&self, lower_bead_count: i64) -> f32 {
        self.parent.transition_anchor_pos(lower_bead_count)
    }

    fn description(&self) -> String {
        format!("RedistributeBeadingStrategy+{}", self.parent.description())
    }
}
