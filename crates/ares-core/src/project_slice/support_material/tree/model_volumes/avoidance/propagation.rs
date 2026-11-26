//! Bottom-up `calculateAvoidance` task kernel; cache identities remain snapped.

use std::ops::Range;

use super::{AvoidanceType, TreeModelVolumes, movement::Movement};
use crate::geometry::simplification::polygons_simplify;
use crate::geometry::{
    ClipperError, Coord, JoinType, Polygon, difference_polygons_paths, offset_paths,
    union_polygons_paths_pair,
};

impl TreeModelVolumes<'_> {
    pub(super) fn propagate_avoidance_cached(
        &self,
        radius: Coord,
        layers: Range<usize>,
        (kind, to_model): (AvoidanceType, bool),
        cancel: &crate::CancelToken,
    ) -> Result<Option<Vec<Vec<Polygon>>>, crate::SliceError> {
        let holefree = kind != AvoidanceType::Fast && radius < self.holefree_limit();
        let maximum = if kind == AvoidanceType::Slow {
            self.max_move_slow
        } else {
            self.max_move
        };
        let Some(movement) = Movement::new(
            radius,
            self.settings.xy_min_distance,
            maximum,
            self.minimum_final_step,
        ) else {
            return Ok(None);
        };
        let err = |error: ClipperError| {
            crate::SliceError::InvalidInput(format!("organic support geometry: {error:?}"))
        };
        let resolution = self.settings.mesh.resolution as f64;
        let radius_key = self.radii.ceil(radius, true);
        let mut latest = if layers.start == 1 {
            self.collision_cache
                .area(radius_key, 0)
                .expect("precalculated layer 0 collision")
                .to_vec()
        } else {
            self.avoidance_caches[usize::from(to_model)][kind.index()]
                .area(radius_key, layers.start - 1)
                .expect("precalculated lower avoidance")
                .to_vec()
        };
        let mut data = Vec::with_capacity(layers.len());
        for layer in layers {
            cancel.check()?;
            let collisions = if holefree {
                self.collision_cache_holefree
                    .area(radius_key, layer)
                    .expect("precalculated holefree collision")
            } else {
                self.collision_cache
                    .area(radius_key, layer)
                    .expect("precalculated collision")
            };
            for step in 0..movement.count {
                latest = union_polygons_paths_pair(
                    collisions,
                    &offset_paths(&latest, movement.offset(step), JoinType::Round, resolution)
                        .map_err(err)?,
                )
                .map_err(err)?;
            }
            if to_model {
                let placeable = self.cached_placeable(radius, radius_key, layer);
                latest = difference_polygons_paths(&latest, placeable).map_err(err)?;
            }
            latest = polygons_simplify(latest, resolution).map_err(err)?;
            data.push(latest.clone());
        }
        Ok(Some(data))
    }

    fn cached_placeable(&self, radius: Coord, radius_key: Coord, layer: usize) -> &[Polygon] {
        if let Some(area) = self.placeable_areas_cache.area(radius_key, layer) {
            return area;
        }
        if radius == 0
            && let Some(area) = self.collision_cache.area(radius_key, layer)
        {
            return area;
        }
        panic!("precalculated placeable area");
    }

    pub(super) fn propagate_avoidance(
        &mut self,
        radius: Coord,
        layers: Range<usize>,
        kind: AvoidanceType,
        to_model: bool,
    ) -> Result<(), ClipperError> {
        if to_model {
            self.placeable_areas(radius, layers.end - 1)?;
        }
        let holefree = kind != AvoidanceType::Fast && radius < self.holefree_limit();
        let maximum = if kind == AvoidanceType::Slow {
            self.max_move_slow
        } else {
            self.max_move
        };
        let Some(movement) = Movement::new(
            radius,
            self.settings.xy_min_distance,
            maximum,
            self.minimum_final_step,
        ) else {
            return Ok(());
        };
        let resolution = self.settings.mesh.resolution as f64;
        let mut latest = self
            .avoidance((radius, layers.start - 1), kind, to_model, true)?
            .to_vec();
        let mut data = Vec::with_capacity(layers.len());
        for layer in layers.clone() {
            let collisions = if holefree {
                self.collision_holefree(radius, layer)?
            } else {
                self.collision(radius, layer, true)?
            };
            for step in 0..movement.count {
                latest = union_polygons_paths_pair(
                    collisions,
                    &offset_paths(&latest, movement.offset(step), JoinType::Round, resolution)?,
                )?;
            }
            if to_model {
                latest = difference_polygons_paths(&latest, self.placeable_areas(radius, layer)?)?;
            }
            latest = polygons_simplify(latest, resolution)?;
            data.push(latest.clone());
        }
        self.avoidance_caches[usize::from(to_model)][kind.index()].insert(
            radius,
            layers.start,
            data,
        );
        Ok(())
    }
}
