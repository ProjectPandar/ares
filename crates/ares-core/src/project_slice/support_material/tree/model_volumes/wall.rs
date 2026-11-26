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

    pub(super) fn calculate_wall_restrictions_par(
        &mut self,
        radius: Coord,
        max_layer: usize,
        cancel: &crate::CancelToken,
    ) -> Result<(), crate::SliceError> {
        let begin = self
            .wall_restrictions_cache
            .max_calculated_layer(radius)
            .unwrap_or(1)
            .max(1);
        let has_minimum_xy = self.settings.xy_distance > self.settings.xy_min_distance;
        let zero_normal = self.radii.ceil(0, false);
        let zero_min = self.radii.ceil(0, true);
        let radius_key = self.radii.ceil(radius, true);
        if self.collision_cache.area(zero_normal, max_layer).is_none() {
            self.calculate_collision_par(zero_normal, max_layer, cancel)?;
        }
        if self
            .collision_cache
            .area(radius_key, max_layer.saturating_sub(1))
            .is_none()
        {
            self.calculate_collision_par(radius_key, max_layer.saturating_sub(1), cancel)?;
        }
        if has_minimum_xy && self.collision_cache.area(zero_min, max_layer).is_none() {
            self.calculate_collision_par(zero_min, max_layer, cancel)?;
        }
        let err = |error: ClipperError| {
            crate::SliceError::InvalidInput(format!("organic support geometry: {error:?}"))
        };
        let shared = &*self;
        let built = crate::parallel::par_range_map_try(max_layer + 1 - begin, cancel, |offset| {
            let layer = begin + offset;
            let normal = shared
                .wall_intersection(radius, layer, false)
                .map_err(err)?;
            let min = if has_minimum_xy {
                Some(shared.wall_intersection(radius, layer, true).map_err(err)?)
            } else {
                None
            };
            Ok((normal, min))
        })?;
        let mut data = Vec::with_capacity(built.len());
        let mut data_min = if has_minimum_xy {
            Vec::with_capacity(built.len())
        } else {
            Vec::new()
        };
        for (normal, min) in built {
            data.push(normal);
            if let Some(min) = min {
                data_min.push(min);
            }
        }
        self.wall_restrictions_cache.insert(radius, begin, data);
        if !data_min.is_empty() {
            self.wall_restrictions_cache_min
                .insert(radius, begin, data_min);
        }
        Ok(())
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
