//! Enabled single-outline collision owner from `TreeModelVolumes.cpp/.hpp`.

mod avoidance;
mod cache;
mod collision;
mod holefree;
mod placeable;
mod precalculate;
mod radius;
mod wall;

use super::super::layer_input::to_polygons;
use super::organic::TreeSettings;
use crate::geometry::simplification::polygons_simplify;
use crate::geometry::{ClipperError, Coord, CoordinateScale, ExPolygon, Polygon};
pub(super) use avoidance::AvoidanceType;
use cache::RadiusLayerPolygonCache;
use radius::RadiusQuantization;

pub(super) struct TreeModelVolumes<'a> {
    settings: &'a TreeSettings,
    outlines: Vec<Vec<Polygon>>,
    pub(super) bed_area: Polygon,
    anti_overhang: Vec<Vec<Polygon>>,
    radii: RadiusQuantization,
    max_move: Coord,
    max_move_slow: Coord,
    minimum_final_step: f32,
    avoidance_caches: [[RadiusLayerPolygonCache; 3]; 2],
    collision_cache: RadiusLayerPolygonCache,
    collision_cache_holefree: RadiusLayerPolygonCache,
    placeable_areas_cache: RadiusLayerPolygonCache,
    wall_restrictions_cache: RadiusLayerPolygonCache,
    wall_restrictions_cache_min: RadiusLayerPolygonCache,
}

impl<'a> TreeModelVolumes<'a> {
    pub(super) fn ceil_radius(&self, radius: Coord, minimum_xy: bool) -> Coord {
        self.radii.ceil(radius, minimum_xy)
    }

    pub(super) fn next_ceil_radius(&self, radius: Coord, minimum_xy: bool) -> Coord {
        self.radii.next_ceil(radius, minimum_xy)
    }

    pub(super) fn new(
        settings: &'a TreeSettings,
        slices: &[Vec<ExPolygon>],
        anti_overhang: Vec<Vec<Polygon>>,
        bed_area: Polygon,
        scale: CoordinateScale,
    ) -> Result<Self, ClipperError> {
        debug_assert!(settings.xy_distance >= settings.xy_min_distance);
        let mut outlines = Vec::with_capacity(settings.raft_layers.len() + slices.len());
        outlines.resize_with(settings.raft_layers.len(), Vec::new);
        for layer in slices {
            outlines.push(polygons_simplify(
                to_polygons(layer),
                settings.mesh.resolution as f64,
            )?);
        }
        Ok(Self {
            settings,
            outlines,
            bed_area,
            anti_overhang,
            radii: RadiusQuantization::new(
                scale,
                settings.radius(0, 0.0),
                settings.xy_distance - settings.xy_min_distance,
            ),
            max_move: (settings.maximum_move_distance - 2).max(0),
            max_move_slow: (settings.maximum_move_distance_slow - 2).max(0),
            minimum_final_step: scale.scaled_delta(0.05) as f32,
            avoidance_caches: std::array::from_fn(|_| std::array::from_fn(|_| Default::default())),
            collision_cache: RadiusLayerPolygonCache::default(),
            collision_cache_holefree: RadiusLayerPolygonCache::default(),
            placeable_areas_cache: RadiusLayerPolygonCache::default(),
            wall_restrictions_cache: RadiusLayerPolygonCache::default(),
            wall_restrictions_cache_min: RadiusLayerPolygonCache::default(),
        })
    }

    pub(super) fn collision(
        &mut self,
        radius: Coord,
        layer: usize,
        minimum_xy: bool,
    ) -> Result<&[Polygon], ClipperError> {
        let radius = self.radii.ceil(radius, minimum_xy);
        if self.collision_cache.area(radius, layer).is_none() {
            self.calculate_collision(radius, layer)?;
        }
        Ok(self
            .collision_cache
            .area(radius, layer)
            .expect("calculated collision inserted every requested layer"))
    }

    pub(super) fn collision_lower_bound_area(
        &self,
        layer: usize,
        max_radius: Coord,
    ) -> Option<(Coord, &[Polygon])> {
        self.collision_cache.lower_bound_area(max_radius, layer)
    }

    pub(super) fn clear_all_but_object_collision(&mut self) {
        // The source intentionally retains the complete collision cache.
        self.collision_cache_holefree = RadiusLayerPolygonCache::default();
        self.avoidance_caches =
            std::array::from_fn(|_| std::array::from_fn(|_| Default::default()));
        self.placeable_areas_cache.clear_all_but_radius_zero();
        self.wall_restrictions_cache = RadiusLayerPolygonCache::default();
        self.wall_restrictions_cache_min = RadiusLayerPolygonCache::default();
    }
}
