//! `TreeModelVolumes::getPlaceableAreas/calculatePlaceables`.

use super::TreeModelVolumes;
use crate::geometry::{
    ClipperError, Coord, FillRule, JoinType, Polygon, difference_polygons_paths,
    offset_expolygons_paths, union_ex,
};

impl TreeModelVolumes<'_> {
    pub(in super::super) fn placeable_areas(
        &mut self,
        original_radius: Coord,
        layer: usize,
    ) -> Result<&[Polygon], ClipperError> {
        let radius = self.radii.ceil(original_radius, true);
        if self.placeable_areas_cache.area(radius, layer).is_none() {
            if original_radius == 0 {
                // Source returns collision on this miss, even if it also filled the landing cache.
                return self.collision(0, layer, true);
            }
            self.calculate_placeables(radius, layer)?;
        }
        Ok(self
            .placeable_areas_cache
            .area(radius, layer)
            .expect("placeable calculation inserted every requested layer"))
    }

    pub(super) fn calculate_placeables_par(
        &mut self,
        radius: Coord,
        max_layer: usize,
        cancel: &crate::CancelToken,
    ) -> Result<(), crate::SliceError> {
        let begin = self
            .placeable_areas_cache
            .max_calculated_layer(radius)
            .map_or(0, |layer| layer + 1);
        if begin > max_layer {
            return Ok(());
        }
        let err = |error: ClipperError| {
            crate::SliceError::InvalidInput(format!("organic support geometry: {error:?}"))
        };
        let zero_key = self.radii.ceil(0, true);
        if begin == 0 && self.collision_cache.area(radius, 0).is_none() {
            self.calculate_collision_par(radius, 0, cancel)?;
        }
        let start = begin.max(1);
        for layer in start..=max_layer {
            if self.placeable_areas_cache.area(zero_key, layer).is_none()
                && self.collision_cache.area(zero_key, layer).is_none()
            {
                self.calculate_collision_par(zero_key, max_layer, cancel)?;
                break;
            }
        }
        let mut data = vec![Vec::new(); max_layer + 1 - begin];
        if begin == 0 {
            let col0 = self
                .collision_cache
                .area(radius, 0)
                .expect("layer 0 collision cached");
            data[0] = difference_polygons_paths(&[], col0).map_err(err)?;
        }
        let distance = -(radius + self.settings.xy_distance) as f32;
        if start <= max_layer {
            let shared = &*self;
            let built =
                crate::parallel::par_range_map_try(max_layer + 1 - start, cancel, |offset| {
                    let layer = start + offset;
                    let base = shared
                        .placeable_areas_cache
                        .area(zero_key, layer)
                        .or_else(|| shared.collision_cache.area(zero_key, layer))
                        .expect("layer 0 placeable or collision cached");
                    offset_expolygons_paths(
                        &union_ex(base, FillRule::NonZero).map_err(err)?,
                        distance,
                        JoinType::Miter,
                        1.2,
                    )
                    .map_err(err)
                })?;
            for (offset, area) in built.into_iter().enumerate() {
                data[start + offset - begin] = area;
            }
        }
        self.placeable_areas_cache.insert(radius, begin, data);
        Ok(())
    }

    pub(super) fn calculate_placeables(
        &mut self,
        radius: Coord,
        max_layer: usize,
    ) -> Result<(), ClipperError> {
        let begin = self
            .placeable_areas_cache
            .max_calculated_layer(radius)
            .map_or(0, |layer| layer + 1);
        if begin > max_layer {
            return Ok(());
        }
        let mut data = vec![Vec::new(); max_layer + 1 - begin];
        if begin == 0 {
            // calculateMachineBorderCollision's enabled branch is empty.
            data[0] = difference_polygons_paths(&[], self.collision(radius, 0, true)?)?;
        }
        let distance = -(radius + self.settings.xy_distance) as f32;
        for layer in begin.max(1)..=max_layer {
            data[layer - begin] = offset_expolygons_paths(
                &union_ex(self.placeable_areas(0, layer)?, FillRule::NonZero)?,
                distance,
                JoinType::Miter,
                1.2,
            )?;
        }
        self.placeable_areas_cache.insert(radius, begin, data);
        Ok(())
    }
}
