//! `TreeModelVolumes.cpp:416-580::calculateCollision`, enabled one-mesh branch.

use super::TreeModelVolumes;
use crate::geometry::{
    ClipperError, Coord, FillRule, JoinType, Polygon, difference_polygons_paths,
    offset_expolygons_paths, offset_paths, simplification::polygons_simplify, union_ex,
    union_polygons_paths,
};

type LayerPolygons = Vec<Vec<Polygon>>;

impl TreeModelVolumes<'_> {
    pub(super) fn calculate_collision_par(
        &mut self,
        radius: Coord,
        max_layer: usize,
        cancel: &crate::CancelToken,
    ) -> Result<(), crate::SliceError> {
        let begin = self
            .collision_cache
            .max_calculated_layer(radius)
            .map_or(0, |layer| layer + 1);
        let end = max_layer + 1;
        let (collisions, placeable) = self.collision_layers_par(radius, begin, end, cancel)?;
        self.collision_cache.insert(radius, begin, collisions);
        if self.settings.support_rests_on_model && radius == 0 {
            self.placeable_areas_cache.insert(radius, begin, placeable);
        }
        Ok(())
    }

    fn collision_layers_par(
        &self,
        radius: Coord,
        begin: usize,
        end: usize,
        cancel: &crate::CancelToken,
    ) -> Result<(LayerPolygons, LayerPolygons), crate::SliceError> {
        let calculate_placeable = self.settings.support_rests_on_model && radius == 0;
        if self.outlines.is_empty() {
            return Ok((
                vec![Vec::new(); end - begin],
                if calculate_placeable {
                    vec![Vec::new(); end - begin]
                } else {
                    Vec::new()
                },
            ));
        }
        let bottom = self.settings.z_distance_bottom_layers;
        let top = self.settings.z_distance_top_layers;
        let xy = self.settings.xy_min_distance;
        let half_layer = if top == 1 { 0.5 } else { 0.0 };
        let offset_begin = begin.saturating_sub(bottom);
        let offset_end = self.outlines.len().min(end + top);
        let offsetted =
            crate::parallel::par_range_map_try(offset_end - offset_begin, cancel, |offset| {
                let layer = offset_begin + offset;
                if radius + xy == 0 {
                    union_polygons_paths(&self.outlines[layer]).map_err(organic_error)
                } else {
                    offset_union(&self.outlines[layer], radius + xy).map_err(organic_error)
                }
            })?;
        let collisions = crate::parallel::par_range_map_try(end - begin, cancel, |index| {
            let layer = begin + index;
            let mut accumulated = Vec::new();
            for below in layer.saturating_sub(bottom).max(offset_begin)..(layer + 1).min(offset_end)
            {
                accumulated.extend_from_slice(&offsetted[below - offset_begin]);
            }
            for distance in 1..=top.min(self.outlines.len().saturating_sub(layer + 1)) {
                let above = layer + distance;
                let required_xy =
                    (xy as f64 - (distance as f64 - half_layer) * xy as f64 / top as f64) as Coord;
                accumulated.extend(
                    offset_union(&self.outlines[above], radius + required_xy)
                        .map_err(organic_error)?,
                );
            }
            polygons_simplify(
                union_polygons_paths(&accumulated).map_err(organic_error)?,
                self.settings.mesh.resolution as f64,
            )
            .map_err(organic_error)
        })?;
        let placeable = if calculate_placeable {
            self.placeable_layers_par(&offsetted, offset_begin, begin..end, cancel)?
        } else {
            Vec::new()
        };
        Ok((collisions, placeable))
    }

    fn placeable_layers_par(
        &self,
        offsetted: &[Vec<Polygon>],
        offset_begin: usize,
        layers: std::ops::Range<usize>,
        cancel: &crate::CancelToken,
    ) -> Result<LayerPolygons, crate::SliceError> {
        let bottom = self.settings.z_distance_bottom_layers;
        let begin = layers.start;
        let end = layers.end;
        let mut placeable = vec![Vec::new(); end - begin];
        let start = (bottom + 1).max(begin);
        if start >= end {
            return Ok(placeable);
        }
        let built = crate::parallel::par_range_map_try(end - start, cancel, |offset| {
            let layer = start + offset;
            let below = layer - bottom - 1;
            let current = &offsetted[layer - offset_begin];
            self.landing_area(below, current).map_err(organic_error)
        })?;
        for (offset, area) in built.into_iter().enumerate() {
            placeable[start + offset - begin] = area;
        }
        Ok(placeable)
    }

    pub(super) fn calculate_collision(
        &mut self,
        radius: Coord,
        max_layer: usize,
    ) -> Result<(), ClipperError> {
        let begin = self
            .collision_cache
            .max_calculated_layer(radius)
            .map_or(0, |layer| layer + 1);
        let end = max_layer + 1;
        let (collisions, placeable) = self.collision_layers(radius, begin, end)?;
        self.collision_cache.insert(radius, begin, collisions);
        if self.settings.support_rests_on_model && radius == 0 {
            self.placeable_areas_cache.insert(radius, begin, placeable);
        }
        Ok(())
    }

    fn collision_layers(
        &self,
        radius: Coord,
        begin: usize,
        end: usize,
    ) -> Result<(LayerPolygons, LayerPolygons), ClipperError> {
        let mut collisions = vec![Vec::new(); end - begin];
        let calculate_placeable = self.settings.support_rests_on_model && radius == 0;
        let mut placeable = if calculate_placeable {
            vec![Vec::new(); end - begin]
        } else {
            Vec::new()
        };
        if self.outlines.is_empty() {
            return Ok((collisions, placeable));
        }
        let bottom = self.settings.z_distance_bottom_layers;
        let top = self.settings.z_distance_top_layers;
        let xy = self.settings.xy_min_distance;
        let half_layer = if top == 1 { 0.5 } else { 0.0 };
        let offset_begin = begin.saturating_sub(bottom);
        let offset_end = self.outlines.len().min(end + top);
        let mut offsetted = Vec::with_capacity(offset_end - offset_begin);
        for layer in offset_begin..offset_end {
            offsetted.push(if radius + xy == 0 {
                union_polygons_paths(&self.outlines[layer])?
            } else {
                offset_union(&self.outlines[layer], radius + xy)?
            });
        }
        for (index, output) in collisions.iter_mut().enumerate() {
            let layer = begin + index;
            let mut accumulated = Vec::new();
            for below in layer.saturating_sub(bottom).max(offset_begin)..(layer + 1).min(offset_end)
            {
                accumulated.extend_from_slice(&offsetted[below - offset_begin]);
            }
            for distance in 1..=top.min(self.outlines.len().saturating_sub(layer + 1)) {
                let above = layer + distance;
                let required_xy =
                    (xy as f64 - (distance as f64 - half_layer) * xy as f64 / top as f64) as Coord;
                accumulated.extend(offset_union(&self.outlines[above], radius + required_xy)?);
            }
            // TreeModelVolumes.cpp simplifies collisions after the last mesh outline.
            *output = polygons_simplify(
                union_polygons_paths(&accumulated)?,
                self.settings.mesh.resolution as f64,
            )?;
        }
        if calculate_placeable {
            for layer in (bottom + 1).max(begin)..end {
                let below = layer - bottom - 1;
                let current = &offsetted[layer - offset_begin];
                placeable[layer - begin] = self.landing_area(below, current)?;
            }
        }
        Ok((collisions, placeable))
    }

    fn landing_area(
        &self,
        below: usize,
        current: &[Polygon],
    ) -> Result<Vec<Polygon>, ClipperError> {
        let inflated = offset_paths(
            &self.outlines[below],
            self.settings.xy_min_distance as f32,
            JoinType::Miter,
            3.0,
        )?;
        if let Some(blockers) = self.anti_overhang.get(below) {
            let mut combined = Vec::with_capacity(current.len() + blockers.len());
            combined.extend_from_slice(current);
            combined.extend_from_slice(blockers);
            difference_polygons_paths(&inflated, &union_polygons_paths(&combined)?)
        } else {
            difference_polygons_paths(&inflated, current)
        }
    }
}

fn organic_error(error: ClipperError) -> crate::SliceError {
    crate::SliceError::InvalidInput(format!("organic support geometry: {error:?}"))
}

fn offset_union(polygons: &[Polygon], distance: Coord) -> Result<Vec<Polygon>, ClipperError> {
    offset_expolygons_paths(
        &union_ex(polygons, FillRule::NonZero)?,
        distance as f32,
        JoinType::Miter,
        1.2,
    )
}
