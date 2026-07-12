use super::PerimeterOptions;

impl PerimeterOptions {
    pub const fn with_wall_transition_length_percent(
        mut self,
        wall_transition_length_percent: f64,
    ) -> Self {
        self.wall_transition_length_percent = wall_transition_length_percent;
        self
    }

    pub const fn with_wall_transition_filter_deviation_percent(
        mut self,
        wall_transition_filter_deviation_percent: f64,
    ) -> Self {
        self.wall_transition_filter_deviation_percent = wall_transition_filter_deviation_percent;
        self
    }

    pub const fn with_wall_transition_angle_degrees(
        mut self,
        wall_transition_angle_degrees: f64,
    ) -> Self {
        self.wall_transition_angle_degrees = wall_transition_angle_degrees;
        self
    }

    pub const fn with_wall_distribution_count(mut self, wall_distribution_count: u32) -> Self {
        self.wall_distribution_count = wall_distribution_count;
        self
    }

    pub const fn with_min_nozzle_diameter(mut self, min_nozzle_diameter: f64) -> Self {
        self.min_nozzle_diameter = min_nozzle_diameter;
        self
    }

    pub const fn with_min_feature_size_percent(mut self, min_feature_size_percent: f64) -> Self {
        self.min_feature_size_percent = min_feature_size_percent;
        self
    }

    pub const fn with_initial_layer_min_bead_width_percent(
        mut self,
        initial_layer_min_bead_width_percent: f64,
    ) -> Self {
        self.initial_layer_min_bead_width_percent = initial_layer_min_bead_width_percent;
        self
    }

    pub const fn with_min_bead_width_percent(mut self, min_bead_width_percent: f64) -> Self {
        self.min_bead_width_percent = min_bead_width_percent;
        self
    }

    pub const fn with_wall_maximum_resolution_mm(
        mut self,
        wall_maximum_resolution_mm: f64,
    ) -> Self {
        self.wall_maximum_resolution_mm = wall_maximum_resolution_mm;
        self
    }

    pub const fn with_wall_maximum_deviation_mm(mut self, wall_maximum_deviation_mm: f64) -> Self {
        self.wall_maximum_deviation_mm = wall_maximum_deviation_mm;
        self
    }

    pub const fn wall_transition_length_percent(&self) -> f64 {
        self.wall_transition_length_percent
    }

    pub const fn wall_transition_filter_deviation_percent(&self) -> f64 {
        self.wall_transition_filter_deviation_percent
    }

    pub fn wall_transition_length_mm(&self) -> f64 {
        self.wall_transition_length_percent / 100.0 * self.min_nozzle_diameter
    }

    pub fn wall_transition_filter_deviation_mm(&self) -> f64 {
        self.wall_transition_filter_deviation_percent / 100.0 * self.min_nozzle_diameter
    }

    pub const fn wall_transition_angle_degrees(&self) -> f64 {
        self.wall_transition_angle_degrees
    }

    pub const fn wall_distribution_count(&self) -> u32 {
        self.wall_distribution_count
    }

    pub const fn min_nozzle_diameter(&self) -> f64 {
        self.min_nozzle_diameter
    }

    pub const fn min_feature_size_percent(&self) -> f64 {
        self.min_feature_size_percent
    }

    pub fn min_feature_size_mm(&self) -> f64 {
        self.min_feature_size_percent / 100.0 * self.min_nozzle_diameter
    }

    pub const fn initial_layer_min_bead_width_percent(&self) -> f64 {
        self.initial_layer_min_bead_width_percent
    }

    pub fn initial_layer_min_bead_width_mm(&self) -> f64 {
        self.initial_layer_min_bead_width_percent / 100.0 * self.min_nozzle_diameter
    }

    pub const fn min_bead_width_percent(&self) -> f64 {
        self.min_bead_width_percent
    }

    pub fn min_bead_width_mm(&self) -> f64 {
        self.min_bead_width_percent / 100.0 * self.min_nozzle_diameter
    }

    pub fn min_bead_width_mm_for_layer(&self, layer_id: usize) -> f64 {
        if layer_id == 0 {
            self.initial_layer_min_bead_width_mm()
        } else {
            self.min_bead_width_mm()
        }
    }

    pub const fn wall_maximum_resolution_mm(&self) -> f64 {
        self.wall_maximum_resolution_mm
    }

    pub const fn wall_maximum_deviation_mm(&self) -> f64 {
        self.wall_maximum_deviation_mm
    }
}
