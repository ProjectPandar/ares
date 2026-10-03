//! `TreeModelVolumes::getWallRestriction/calculateWallRestrictions`.

use super::TreeModelVolumes;
use crate::geometry::simplification::polygons_simplify;
use crate::geometry::{ClipperError, Coord, Polygon, intersection_polygons_paths};

impl TreeModelVolumes<'_> {
    pub(in super::super) fn wall_restriction(
        &mut self,
        original_radius: Coord,
        layer: usize,
        minimum_xy: bool,
    ) -> Result<&[Polygon], ClipperError> {
        debug_assert!(layer > 0);
        if layer == 0 {
            return self.collision(original_radius, layer, minimum_xy);
        }
        let minimum_xy = minimum_xy && self.settings.xy_distance > self.settings.xy_min_distance;
        let radius = self.radii.ceil(original_radius, true);
        if self.wall_cache(minimum_xy).area(radius, layer).is_none() {
            self.calculate_wall_restrictions(radius, layer)?;
        }
        Ok(self
            .wall_cache(minimum_xy)
            .area(radius, layer)
            .expect("wall calculation inserted every requested layer"))
    }

    fn wall_cache(&self, minimum_xy: bool) -> &super::cache::RadiusLayerPolygonCache {
        if minimum_xy {
            &self.wall_restrictions_cache_min
        } else {
            &self.wall_restrictions_cache
        }
    }

    pub(super) fn calculate_wall_restrictions(
        &mut self,
        radius: Coord,
        max_layer: usize,
    ) -> Result<(), ClipperError> {
        // Upstream deliberately recomputes the highest cached layer, not the next.
        let begin = self
            .wall_restrictions_cache
            .max_calculated_layer(radius)
            .unwrap_or(1)
            .max(1);
        let has_minimum_xy = self.settings.xy_distance > self.settings.xy_min_distance;
        let mut data = Vec::with_capacity(max_layer + 1 - begin);
        let mut data_min = if has_minimum_xy {
            Vec::with_capacity(max_layer + 1 - begin)
        } else {
            Vec::new()
        };
        for layer in begin..=max_layer {
            self.collision(0, layer, false)?;
            self.collision(radius, layer - 1, true)?;
            data.push(self.wall_intersection(radius, layer, false)?);
            if has_minimum_xy {
                self.collision(0, layer, true)?;
                data_min.push(self.wall_intersection(radius, layer, true)?);
            }
        }
        self.wall_restrictions_cache.insert(radius, begin, data);
        if !data_min.is_empty() {
            self.wall_restrictions_cache_min
                .insert(radius, begin, data_min);
        }
        Ok(())
    }

    fn wall_intersection(
        &self,
        radius: Coord,
        layer: usize,
        minimum_xy: bool,
    ) -> Result<Vec<Polygon>, ClipperError> {
        let upper = self
            .collision_cache
            .area(self.radii.ceil(0, minimum_xy), layer)
            .expect("upper collision was calculated");
        let lower = self
            .collision_cache
            .area(self.radii.ceil(radius, true), layer - 1)
            .expect("lower collision was calculated");
        polygons_simplify(
            intersection_polygons_paths(upper, lower)?,
            self.settings.mesh.resolution as f64,
        )
    }
}
