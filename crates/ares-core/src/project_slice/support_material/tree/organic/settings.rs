//! `TreeSupportSettings` derived configuration and radius calculations.

use super::{MeshGroupSettings, raft};
use crate::{
    geometry::{Coord, CoordinateScale},
    project_slice::parameters::SlicingParameters,
};

pub(in super::super) struct TreeSettings {
    pub(in super::super) mesh: MeshGroupSettings,
    pub(in super::super) branch_radius: Coord,
    pub(in super::super) min_radius: Coord,
    pub(in super::super) maximum_move_distance: Coord,
    pub(in super::super) maximum_move_distance_slow: Coord,
    pub(in super::super) tip_layers: usize,
    pub(in super::super) branch_radius_increase_per_layer: f64,
    pub(in super::super) max_to_model_radius_increase: Coord,
    pub(in super::super) min_dtt_to_model: usize,
    pub(in super::super) increase_radius_until_radius: Coord,
    pub(in super::super) increase_radius_until_layer: usize,
    pub(in super::super) support_rests_on_model: bool,
    pub(in super::super) xy_distance: Coord,
    pub(in super::super) xy_min_distance: Coord,
    pub(in super::super) buildplate_radius_increase_per_layer: f64,
    pub(in super::super) layer_start_buildplate_radius: i32,
    pub(in super::super) z_distance_bottom_layers: usize,
    pub(in super::super) z_distance_top_layers: usize,
    pub(in super::super) raft_layers: Vec<f64>,
}

impl TreeSettings {
    pub(in super::super) fn new(
        mesh: MeshGroupSettings,
        slicing: &SlicingParameters,
        zero_top_z_gap: bool,
        scale: CoordinateScale,
    ) -> Self {
        let layer_height = mesh.layer_height;
        let branch_radius = mesh.branch_diameter / 2;
        let min_radius = mesh.tip_diameter / 2;
        let tip_layers = ((branch_radius - min_radius) / (mesh.support_line_width / 3))
            .max(branch_radius / layer_height) as usize;
        debug_assert!(tip_layers > 0);
        let branch_radius_increase_per_layer = mesh.diameter_angle.tan() * layer_height as f64;
        let buildplate_radius = mesh.buildplate_diameter / 2;
        let buildplate_radius_increase_per_layer =
            (0.7_f64.tan() * layer_height as f64).min(0.5 * mesh.support_line_width as f64);
        let layer_start_buildplate_radius = ((buildplate_radius - branch_radius) as f64
            / buildplate_radius_increase_per_layer)
            as i32;
        let mut xy_distance = mesh.xy_distance;
        let mut xy_min_distance = mesh.xy_distance.min(mesh.overhang_xy_distance);
        if zero_top_z_gap {
            xy_min_distance = xy_min_distance.max(scale.scaled_delta(0.1) as Coord);
            xy_distance = xy_distance.max(xy_min_distance);
        }
        Self {
            branch_radius,
            min_radius,
            maximum_move_distance: (mesh.branch_angle.tan() * layer_height as f64) as Coord,
            maximum_move_distance_slow: (mesh.slow_angle.tan() * layer_height as f64) as Coord,
            tip_layers,
            branch_radius_increase_per_layer,
            max_to_model_radius_increase: mesh.max_diameter_increase_to_model / 2,
            min_dtt_to_model: ((mesh.min_height_to_model + layer_height - 1) / layer_height)
                as usize,
            increase_radius_until_radius: branch_radius,
            increase_radius_until_layer: tip_layers,
            support_rests_on_model: !mesh.buildplate_only,
            xy_distance,
            xy_min_distance,
            buildplate_radius_increase_per_layer,
            layer_start_buildplate_radius,
            z_distance_bottom_layers: (mesh.bottom_distance as f64 / layer_height as f64).round()
                as usize,
            z_distance_top_layers: (mesh.top_distance as f64 / layer_height as f64).round()
                as usize,
            raft_layers: raft::layer_z(slicing),
            mesh,
        }
    }

    pub(in super::super) fn radius(
        &self,
        distance_to_top: usize,
        elephant_foot_increases: f64,
    ) -> Coord {
        let radius = if distance_to_top <= self.tip_layers {
            let distance = i64::try_from(distance_to_top).expect("tip distance fits coordinates");
            let tip_layers = i64::try_from(self.tip_layers).expect("tip count fits coordinates");
            (self.min_radius + (self.branch_radius - self.min_radius) * distance / tip_layers)
                as f64
        } else {
            self.branch_radius as f64
                + (distance_to_top - self.tip_layers) as f64 * self.branch_radius_increase_per_layer
        };
        (radius
            + elephant_foot_increases
                * (self.buildplate_radius_increase_per_layer
                    - self.branch_radius_increase_per_layer)
                    .max(0.0)) as Coord
    }

    pub(in super::super) fn recommended_min_radius(&self, layer_index: i32) -> Coord {
        let widened_layers = f64::from(self.layer_start_buildplate_radius - layer_index);
        if widened_layers > 0.0 {
            (self.branch_radius as f64 + widened_layers * self.buildplate_radius_increase_per_layer)
                as Coord
        } else {
            0
        }
    }
}
