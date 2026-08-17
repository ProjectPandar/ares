use super::base::{Beading, BeadingStrategy, BeadingStrategyConfig};

pub(crate) struct OuterWallInsetBeadingStrategy {
    config: BeadingStrategyConfig,
    parent: Box<dyn BeadingStrategy>,
    outer_wall_offset: i64,
}

impl OuterWallInsetBeadingStrategy {
    pub(crate) fn new(outer_wall_offset: i64, parent: Box<dyn BeadingStrategy>) -> Self {
        Self {
            config: *parent.config(),
            parent,
            outer_wall_offset,
        }
    }
}

impl BeadingStrategy for OuterWallInsetBeadingStrategy {
    fn config(&self) -> &BeadingStrategyConfig {
        &self.config
    }

    fn compute(&self, thickness: i64, bead_count: i64) -> Beading {
        let mut result = self.parent.compute(thickness, bead_count);
        let actual_count = result
            .bead_widths
            .iter()
            .filter(|width| **width > 0)
            .count();
        if actual_count >= 2 {
            result.toolpath_locations[0] =
                (result.toolpath_locations[0] + self.outer_wall_offset).min(thickness / 2);
        }
        result
    }

    fn optimal_thickness(&self, bead_count: i64) -> i64 {
        self.parent.optimal_thickness(bead_count)
    }

    fn transition_thickness(&self, lower_bead_count: i64) -> i64 {
        self.parent.transition_thickness(lower_bead_count)
    }

    fn optimal_bead_count(&self, thickness: i64) -> i64 {
        self.parent.optimal_bead_count(thickness)
    }

    fn transitioning_length(&self, lower_bead_count: i64) -> i64 {
        self.parent.transitioning_length(lower_bead_count)
    }

    fn nonlinear_thicknesses(&self, lower_bead_count: i64) -> Vec<i64> {
        self.parent.nonlinear_thicknesses(lower_bead_count)
    }

    fn description(&self) -> String {
        format!(
            "OuterWallOfsetBeadingStrategy+{}",
            self.parent.description()
        )
    }
}
