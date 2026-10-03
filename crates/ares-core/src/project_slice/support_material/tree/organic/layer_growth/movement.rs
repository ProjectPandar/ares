//! Projected-radius speeds and lazy offsets in `increase_areas_one_layer`.

use super::super::area_growth::{Expansion, safe_offset_inc};
use super::{SupportElement, TreeModelVolumes, TreeSettings};
use crate::geometry::{ClipperError, Coord, CoordinateScale, Polygon};

pub(super) struct Movement {
    pub(super) fast_speed: Coord,
    pub(super) slow_speed: Coord,
    safe_step: Coord,
    radius: Coord,
    independent: bool,
    slow: Vec<Polygon>,
    fast: Vec<Polygon>,
}

impl Movement {
    pub(super) fn new(
        volumes: &TreeModelVolumes<'_>,
        config: &TreeSettings,
        parent: &SupportElement,
        layer: usize,
        scale: CoordinateScale,
    ) -> Self {
        let state = parent.state.propagate_down();
        let radius = state.collision_radius(config);
        let parent_radius = parent.state.collision_radius(config);
        let ceiled_parent = volumes.ceil_radius(parent_radius, parent.state.bits.use_min_xy_dist);
        let projected = config.radius(
            (parent.state.effective_radius_height + 1) as usize,
            parent.state.elephant_foot_increases,
        );
        let delta = projected - parent_radius;
        let mut extra = 5;
        let mut extra_slow = 0;
        if ceiled_parent == volumes.ceil_radius(projected, parent.state.bits.use_min_xy_dist)
            || projected < config.increase_radius_until_radius
        {
            extra += delta;
        } else {
            extra_slow += delta.min(
                config.maximum_move_distance + extra
                    - (config.maximum_move_distance_slow + extra_slow),
            );
        }
        if config.layer_start_buildplate_radius > layer as i32
            && config.recommended_min_radius((layer - 1) as i32)
                < config.radius(
                    (state.effective_radius_height + 1) as usize,
                    state.elephant_foot_increases,
                )
        {
            let foot = config.radius(
                (parent.state.effective_radius_height + 1) as usize,
                parent.state.elephant_foot_increases + 1.0,
            );
            if ceiled_parent == volumes.ceil_radius(foot, parent.state.bits.use_min_xy_dist) {
                // Native compound assignment adds in double before narrowing to Coord.
                extra = (extra as f64 + config.buildplate_radius_increase_per_layer) as Coord;
            } else {
                extra_slow += (config.buildplate_radius_increase_per_layer as Coord).min(
                    config.maximum_move_distance - (config.maximum_move_distance_slow + extra_slow),
                );
            }
        }
        let mut safe_step = if state.bits.use_min_xy_dist {
            config.xy_min_distance
        } else {
            config.xy_distance
        } + if config
            .z_distance_top_layers
            .min(config.z_distance_bottom_layers)
            > 0
        {
            config.mesh.min_feature_size
        } else {
            0
        };
        if safe_step == 0 {
            safe_step = scale.scaled_delta(0.1) as Coord;
        }
        let fast_speed = config.maximum_move_distance + extra;
        let slow_speed = config.maximum_move_distance_slow + extra + extra_slow;
        let independent = radius / safe_step - Coord::from(fast_speed < radius + safe_step)
            > (slow_speed + safe_step - 1) / safe_step;
        Self {
            fast_speed,
            slow_speed,
            safe_step,
            radius,
            independent,
            slow: Vec::new(),
            fast: Vec::new(),
        }
    }

    pub(super) fn offset(
        &mut self,
        parent: &[Polygon],
        wall: &[Polygon],
        speed: Coord,
        scale: CoordinateScale,
    ) -> Result<(), ClipperError> {
        if self.slow.is_empty() && (speed == self.slow_speed || !self.independent) {
            self.slow = safe_offset_inc(
                parent,
                wall,
                Expansion {
                    distance: self.slow_speed,
                    safe_step: self.safe_step,
                    unchecked_last: if self.independent {
                        self.safe_step + self.radius
                    } else {
                        0
                    },
                    minimum_offsets: 2,
                },
                scale,
            )?;
        }
        if self.fast.is_empty() && speed != self.slow_speed {
            self.fast = safe_offset_inc(
                if self.independent { parent } else { &self.slow },
                wall,
                Expansion {
                    distance: if self.independent {
                        self.fast_speed
                    } else {
                        self.fast_speed - self.slow_speed
                    },
                    safe_step: self.safe_step,
                    unchecked_last: self.safe_step + self.radius,
                    minimum_offsets: 1,
                },
                scale,
            )?;
        }
        Ok(())
    }

    pub(super) fn selected(&self, speed: Coord) -> &[Polygon] {
        if speed == self.slow_speed {
            &self.slow
        } else {
            &self.fast
        }
    }
}
