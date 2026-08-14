use crate::geometry::CoordinateScale;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Beading {
    pub(crate) total_thickness: i64,
    pub(crate) bead_widths: Vec<i64>,
    pub(crate) toolpath_locations: Vec<i64>,
    pub(crate) left_over: i64,
}

impl Beading {
    pub(crate) const fn empty(total_thickness: i64) -> Self {
        Self {
            total_thickness,
            bead_widths: Vec::new(),
            toolpath_locations: Vec::new(),
            left_over: total_thickness,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct BeadingStrategyConfig {
    pub(crate) optimal_width: i64,
    pub(crate) wall_split_middle_threshold: f64,
    pub(crate) wall_add_middle_threshold: f64,
    pub(crate) default_transition_length: i64,
    pub(crate) transitioning_angle: f64,
    pub(crate) coordinate_scale: CoordinateScale,
}

pub(crate) trait BeadingStrategy {
    fn config(&self) -> &BeadingStrategyConfig;

    fn compute(&self, thickness: i64, bead_count: i64) -> Beading;

    fn optimal_thickness(&self, bead_count: i64) -> i64 {
        self.config().optimal_width * bead_count
    }

    fn transition_thickness(&self, lower_bead_count: i64) -> i64 {
        let lower = self.optimal_thickness(lower_bead_count);
        let higher = self.optimal_thickness(lower_bead_count + 1);
        let threshold = if lower_bead_count % 2 == 1 {
            self.config().wall_split_middle_threshold
        } else {
            self.config().wall_add_middle_threshold
        };
        (lower as f64 + threshold * (higher - lower) as f64) as i64
    }

    fn optimal_bead_count(&self, thickness: i64) -> i64;

    fn transitioning_length(&self, lower_bead_count: i64) -> i64 {
        if lower_bead_count == 0 {
            self.config().coordinate_scale.checked_scale(0.01).unwrap()
        } else {
            self.config().default_transition_length
        }
    }

    fn transition_anchor_pos(&self, lower_bead_count: i64) -> f32 {
        let lower = self.optimal_thickness(lower_bead_count);
        let transition = self.transition_thickness(lower_bead_count);
        let upper = self.optimal_thickness(lower_bead_count + 1);
        1.0 - (transition - lower) as f32 / (upper - lower) as f32
    }

    fn nonlinear_thicknesses(&self, _lower_bead_count: i64) -> Vec<i64> {
        Vec::new()
    }

    fn description(&self) -> String;

    fn split_middle_threshold(&self) -> f64 {
        self.config().wall_split_middle_threshold
    }

    fn transitioning_angle(&self) -> f64 {
        self.config().transitioning_angle
    }
}
