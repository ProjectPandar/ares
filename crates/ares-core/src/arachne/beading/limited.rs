use super::base::{Beading, BeadingStrategy, BeadingStrategyConfig};

pub(crate) struct LimitedBeadingStrategy {
    config: BeadingStrategyConfig,
    max_bead_count: i64,
    parent: Box<dyn BeadingStrategy>,
}

impl LimitedBeadingStrategy {
    pub(crate) fn new(max_bead_count: i64, parent: Box<dyn BeadingStrategy>) -> Self {
        Self {
            config: *parent.config(),
            max_bead_count,
            parent,
        }
    }

    fn insert_left_marker(&self, result: &mut Beading) {
        let index = (self.max_bead_count / 2) as usize;
        let inner_index = index - 1;
        let location = result.toolpath_locations[inner_index];
        let width = result.bead_widths[inner_index];
        result
            .toolpath_locations
            .insert(index, location + width / 2);
        result.bead_widths.insert(index, 0);
    }
}

impl BeadingStrategy for LimitedBeadingStrategy {
    fn config(&self) -> &BeadingStrategyConfig {
        &self.config
    }

    fn compute(&self, thickness: i64, bead_count: i64) -> Beading {
        if bead_count <= self.max_bead_count {
            let mut result = self.parent.compute(thickness, bead_count);
            let actual_count = result.toolpath_locations.len() as i64;
            if actual_count % 2 == 0 && actual_count == self.max_bead_count {
                self.insert_left_marker(&mut result);
            }
            return result;
        }

        assert_eq!(bead_count, self.max_bead_count + 1);
        let optimal_thickness = self.parent.optimal_thickness(self.max_bead_count);
        let mut result = self.parent.compute(optimal_thickness, self.max_bead_count);
        let actual_count = result.toolpath_locations.len();
        result.left_over += thickness - result.total_thickness;
        result.total_thickness = thickness;

        if actual_count % 2 == 1 {
            result.toolpath_locations[actual_count / 2] = thickness / 2;
            result.bead_widths[actual_count / 2] = thickness - optimal_thickness;
        }
        for index in 0..actual_count.div_ceil(2) {
            result.toolpath_locations[actual_count - 1 - index] =
                thickness - result.toolpath_locations[index];
        }

        self.insert_left_marker(&mut result);
        let opposite_index = actual_count - (self.max_bead_count as usize / 2 - 1);
        let location = result.toolpath_locations[opposite_index];
        let width = result.bead_widths[opposite_index];
        result
            .toolpath_locations
            .insert(opposite_index, location - width / 2);
        result.bead_widths.insert(opposite_index, 0);
        result
    }

    fn optimal_thickness(&self, bead_count: i64) -> i64 {
        assert!(bead_count <= self.max_bead_count);
        self.parent.optimal_thickness(bead_count)
    }

    fn transition_thickness(&self, lower_bead_count: i64) -> i64 {
        if lower_bead_count < self.max_bead_count {
            return self.parent.transition_thickness(lower_bead_count);
        }
        assert_eq!(lower_bead_count, self.max_bead_count);
        self.parent.optimal_thickness(lower_bead_count + 1)
            - self.config.coordinate_scale.checked_scale(0.01).unwrap()
    }

    fn optimal_bead_count(&self, thickness: i64) -> i64 {
        let parent_count = self.parent.optimal_bead_count(thickness);
        if parent_count <= self.max_bead_count {
            return self.parent.optimal_bead_count(thickness);
        }
        if parent_count == self.max_bead_count + 1
            && thickness
                < self.parent.optimal_thickness(self.max_bead_count + 1)
                    - self.config.coordinate_scale.checked_scale(0.01).unwrap()
        {
            self.max_bead_count
        } else {
            self.max_bead_count + 1
        }
    }

    fn transitioning_length(&self, lower_bead_count: i64) -> i64 {
        self.parent.transitioning_length(lower_bead_count)
    }

    fn transition_anchor_pos(&self, lower_bead_count: i64) -> f32 {
        self.parent.transition_anchor_pos(lower_bead_count)
    }

    fn description(&self) -> String {
        format!("LimitedBeadingStrategy+{}", self.parent.description())
    }
}
