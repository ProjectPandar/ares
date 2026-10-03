//! `TreeModelVolumes.cpp:158-253::precalculate` dependency preparation.

use std::collections::BTreeMap;

use super::TreeModelVolumes;
use crate::geometry::ClipperError;

impl TreeModelVolumes<'_> {
    pub(in super::super) fn precalculate(&mut self, max_layer: usize) -> Result<(), ClipperError> {
        self.radii.prepare_tip_radii(self.settings);
        let delta = self.settings.xy_distance - self.settings.xy_min_distance;
        let mut radius_until_layer = BTreeMap::new();
        for distance in 0..=max_layer {
            let layer = max_layer - distance;
            let radius = self.settings.radius(distance, 0.0);
            for radius in [
                self.radii.ceil(radius + delta, true),
                self.radii.ceil(radius, true),
                self.radii.ceil(
                    self.settings.recommended_min_radius(layer as i32) + delta,
                    true,
                ),
            ] {
                radius_until_layer.entry(radius).or_insert(layer);
            }
        }
        let avoidance_keys: Vec<_> = radius_until_layer.iter().map(|(&r, &l)| (r, l)).collect();
        radius_until_layer.insert(
            self.radii
                .ceil(self.settings.increase_radius_until_radius + delta, true),
            max_layer,
        );
        radius_until_layer.insert(0, max_layer);
        if delta != 0 {
            radius_until_layer.insert(delta, max_layer);
        }
        for (radius, layer) in radius_until_layer {
            self.calculate_collision(radius, layer)?;
        }
        for &(radius, layer) in &avoidance_keys {
            if radius < self.holefree_limit() {
                self.calculate_collision_holefree(radius, layer)?;
            }
        }
        if self.settings.support_rests_on_model {
            for &(radius, layer) in &avoidance_keys {
                self.calculate_placeables(radius, layer)?;
            }
        }
        self.calculate_avoidance_batch(
            &avoidance_keys,
            true,
            self.settings.support_rests_on_model,
        )?;
        for (radius, layer) in avoidance_keys {
            self.calculate_wall_restrictions(radius, layer)?;
        }
        Ok(())
    }
}
