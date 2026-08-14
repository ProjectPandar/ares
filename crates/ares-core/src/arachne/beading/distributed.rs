use super::base::{Beading, BeadingStrategy, BeadingStrategyConfig};

pub(crate) struct DistributedBeadingStrategy {
    config: BeadingStrategyConfig,
    one_over_distribution_radius_squared: f32,
}

impl DistributedBeadingStrategy {
    pub(crate) fn new(config: BeadingStrategyConfig, distribution_radius: i32) -> Self {
        let radius = if distribution_radius >= 2 {
            (distribution_radius - 1) as f32
        } else {
            1.0
        };
        Self {
            config,
            one_over_distribution_radius_squared: 1.0 / radius * 1.0 / radius,
        }
    }

    fn compute_many(&self, thickness: i64, bead_count: i64) -> Beading {
        let divided = thickness - bead_count * self.config.optimal_width;
        let middle = (bead_count - 1) as f32 / 2.0;
        let weights = (0..bead_count)
            .map(|index| {
                let deviation = index as f32 - middle;
                (1.0 - self.one_over_distribution_radius_squared * deviation * deviation).max(0.0)
            })
            .collect::<Vec<_>>();
        let total_weight: f32 = weights.iter().sum();
        let mut result = Beading::empty(thickness);
        let mut accumulated_width = 0;
        for (index, weight) in weights.into_iter().enumerate() {
            let width = if index + 1 == bead_count as usize {
                thickness - accumulated_width
            } else {
                self.config.optimal_width + (divided as f32 * (weight / total_weight)) as i64
            };
            let location = result.toolpath_locations.last().map_or(width / 2, |last| {
                *last + (*result.bead_widths.last().unwrap() + width) / 2
            });
            result.toolpath_locations.push(location);
            result.bead_widths.push(width);
            accumulated_width += width;
        }
        result.left_over = 0;
        result
    }
}

impl BeadingStrategy for DistributedBeadingStrategy {
    fn config(&self) -> &BeadingStrategyConfig {
        &self.config
    }

    fn compute(&self, thickness: i64, bead_count: i64) -> Beading {
        let mut result = Beading::empty(thickness);
        match bead_count {
            count if count > 2 => return self.compute_many(thickness, bead_count),
            2 => {
                let outer_width = thickness / 2;
                result.bead_widths = vec![outer_width, outer_width];
                result.toolpath_locations = vec![outer_width / 2, thickness - outer_width / 2];
                result.left_over = 0;
            }
            1 => {
                result.bead_widths.push(thickness);
                result.toolpath_locations.push(thickness / 2);
                result.left_over = 0;
            }
            _ => {}
        }
        result
    }

    fn optimal_bead_count(&self, thickness: i64) -> i64 {
        let naive_count = thickness / self.config.optimal_width;
        let remainder = thickness - naive_count * self.config.optimal_width;
        let threshold = if naive_count % 2 == 1 {
            self.config.wall_split_middle_threshold
        } else {
            self.config.wall_add_middle_threshold
        };
        let minimum_line_width = (self.config.optimal_width as f64 * threshold) as i64;
        naive_count + (remainder >= minimum_line_width) as i64
    }

    fn description(&self) -> String {
        "DistributedBeadingStrategy".into()
    }
}
