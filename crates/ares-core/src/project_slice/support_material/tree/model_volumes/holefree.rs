//! `TreeModelVolumes::getCollisionHolefree/calculateCollisionHolefree`.

use super::TreeModelVolumes;
use crate::geometry::simplification::polygons_simplify;
use crate::geometry::{
    ClipperError, Coord, FillRule, JoinType, Polygon, offset_expolygons_paths, union_ex,
};

impl TreeModelVolumes<'_> {
    pub(super) fn collision_holefree(
        &mut self,
        radius: Coord,
        layer: usize,
    ) -> Result<&[Polygon], ClipperError> {
        debug_assert_eq!(radius, self.radii.ceil(radius, true));
        debug_assert!(
            radius
                < self.settings.increase_radius_until_radius + self.settings.xy_distance
                    - self.settings.xy_min_distance
        );
        if self.collision_cache_holefree.area(radius, layer).is_none() {
            self.calculate_collision_holefree(radius, layer)?;
        }
        Ok(self
            .collision_cache_holefree
            .area(radius, layer)
            .expect("holefree calculation inserted every requested layer"))
    }

    pub(super) fn calculate_collision_holefree(
        &mut self,
        radius: Coord,
        max_layer: usize,
    ) -> Result<(), ClipperError> {
        let increase_radius = self.settings.increase_radius_until_radius;
        let increase_radius_ceil = self.radii.ceil(increase_radius, false) - radius;
        debug_assert!(increase_radius_ceil > 0);
        let resolution = self.settings.mesh.resolution as f64;
        let mut data = Vec::with_capacity(max_layer + 1);
        for layer in 0..=max_layer {
            let collision = union_ex(
                self.collision(increase_radius, layer, false)?,
                FillRule::NonZero,
            )?;
            // Upstream uses five coordinate units, not five scaled millimeters.
            let offset = offset_expolygons_paths(
                &collision,
                (5 - increase_radius_ceil) as f32,
                JoinType::Round,
                resolution,
            )?;
            data.push(polygons_simplify(offset, resolution)?);
        }
        self.collision_cache_holefree.insert(radius, 0, data);
        Ok(())
    }
}
