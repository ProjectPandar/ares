//! Tip state from `TreeSupport3D.hpp::SupportElementState/SupportElement`.

use super::super::model_volumes::AvoidanceType;
use crate::geometry::{Coord, Point, Polygon};

#[derive(Clone, Copy, Default)]
pub(super) struct StateBits {
    pub(super) to_buildplate: bool,
    pub(super) to_model_gracious: bool,
    pub(super) use_min_xy_dist: bool,
    pub(super) supports_roof: bool,
    pub(super) can_use_safe_radius: bool,
    pub(super) skip_ovalisation: bool,
    pub(super) lost: bool,
    pub(super) very_lost: bool,
    pub(super) deleted: bool,
    pub(super) marked: bool,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) struct AreaIncreaseSettings {
    pub(super) increase_speed: Coord,
    pub(super) kind: AvoidanceType,
    pub(super) increase_radius: bool,
    pub(super) no_error: bool,
    pub(super) use_min_distance: bool,
    pub(super) move_area: bool,
}

impl Default for AreaIncreaseSettings {
    fn default() -> Self {
        Self {
            increase_speed: 0,
            kind: AvoidanceType::Fast,
            increase_radius: false,
            no_error: false,
            use_min_distance: false,
            move_area: false,
        }
    }
}

#[derive(Clone, Copy)]
pub(super) struct SupportElementState {
    pub(super) bits: StateBits,
    pub(super) target_height: usize,
    pub(super) target_position: Point,
    pub(super) next_position: Point,
    pub(super) layer_index: usize,
    pub(super) effective_radius_height: u32,
    pub(super) distance_to_top: u32,
    pub(super) result_on_layer: Option<Point>,
    pub(super) increased_to_model_radius: Coord,
    pub(super) elephant_foot_increases: f64,
    pub(super) dont_move_until: u32,
    pub(super) last_area_increase: AreaIncreaseSettings,
    pub(super) missing_roof_layers: u32,
    pub(super) roof_recovery_depth: u32,
}

pub(super) struct SupportElement {
    pub(super) state: SupportElementState,
    pub(super) parents: Vec<i32>,
    pub(super) influence_area: Vec<Polygon>,
}

impl SupportElementState {
    pub(super) fn propagate_down(mut self) -> Self {
        self.distance_to_top += 1;
        self.layer_index -= 1;
        if self.missing_roof_layers > 0 {
            self.missing_roof_layers -= 1;
            self.roof_recovery_depth += 1;
        }
        self.result_on_layer = None;
        self.bits.skip_ovalisation = false;
        self
    }

    pub(super) fn actual_radius(&self, settings: &super::TreeSettings) -> Coord {
        let effective_height = self.effective_radius_height as usize;
        let distance = if effective_height < settings.increase_radius_until_layer {
            (self.distance_to_top as usize).min(settings.increase_radius_until_layer)
        } else {
            effective_height
        };
        settings.radius(distance, self.elephant_foot_increases)
    }

    pub(super) fn collision_radius(&self, settings: &super::TreeSettings) -> Coord {
        settings.radius(
            self.effective_radius_height as usize,
            self.elephant_foot_increases,
        )
    }
}
