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
