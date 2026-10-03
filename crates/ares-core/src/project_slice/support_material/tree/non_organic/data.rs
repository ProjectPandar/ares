//! `TreeSupportData` geometry ownership (`TreeSupport.cpp:3650-3794`).

use std::collections::HashMap;

use super::super::super::layer_input::LayerInput;
use crate::geometry::simplification::append_simplified_expolygon;
use crate::geometry::{
    ClipperError, CoordinateScale, ExPolygon, JoinType, offset_expolygons, union_expolygons,
};

type RadiusLayer = (u64, usize);

pub(super) struct TreeSupportData {
    pub(super) outlines: Vec<Vec<ExPolygon>>,
    pub(super) outlines_below: Vec<Vec<ExPolygon>>,
    max_move_distances: Vec<f64>,
    pub(super) xy_distance: f64,
    radius_resolution: f64,
    scale: CoordinateScale,
    collision_cache: HashMap<RadiusLayer, Vec<ExPolygon>>,
    avoidance_cache: HashMap<RadiusLayer, Vec<ExPolygon>>,
}

impl TreeSupportData {
    pub(super) fn new(
        layers: &[LayerInput<'_>],
        branch_angle_degrees: f64,
        xy_distance: f64,
        radius_resolution: f64,
        scale: CoordinateScale,
    ) -> Result<Self, ClipperError> {
        let branch_scale = (branch_angle_degrees * std::f64::consts::PI / 180.0).tan();
        let tolerance = scale.scaled_delta(radius_resolution);
        let mut outlines = Vec::with_capacity(layers.len());
        let mut outlines_below: Vec<Vec<ExPolygon>> = Vec::with_capacity(layers.len());
        for layer in layers {
            let mut outline = Vec::with_capacity(layer.slices.len());
            for polygon in layer.slices {
                append_simplified_expolygon(polygon.clone(), tolerance, &mut outline)?;
            }
            outlines_below.push(if let Some(previous) = outlines_below.last() {
                union_expolygons(previous, &outline)?
            } else {
                outline.clone()
            });
            outlines.push(outline);
        }
        Ok(Self {
            outlines,
            outlines_below,
            max_move_distances: layers
                .iter()
                .map(|layer| layer.height * branch_scale)
                .collect(),
            xy_distance,
            radius_resolution,
            scale,
            collision_cache: HashMap::new(),
            avoidance_cache: HashMap::new(),
        })
    }

    pub(super) fn ceil_radius(&self, radius: f64) -> f64 {
        let factor = (radius / self.radius_resolution) as usize;
        let remains = radius - self.radius_resolution * factor as f64;
        if remains > 0.0001 {
            radius + self.radius_resolution - remains
        } else {
            radius
        }
    }

    pub(super) fn collision(
        &mut self,
        radius: f64,
        layer: usize,
    ) -> Result<&[ExPolygon], ClipperError> {
        let radius = self.ceil_radius(radius);
        let key = (radius_key(radius), layer);
        if !self.collision_cache.contains_key(&key) {
            let collision = offset_expolygons(
                &self.outlines[layer],
                self.scale.scaled_delta(radius + self.xy_distance) as f32,
                JoinType::Miter,
                3.0,
            )?;
            let mut simplified = Vec::with_capacity(collision.len());
            for polygon in collision {
                append_simplified_expolygon(
                    polygon,
                    self.scale.scaled_delta(self.radius_resolution),
                    &mut simplified,
                )?;
            }
            self.collision_cache.entry(key).or_insert(simplified);
        }
        Ok(self.collision_cache[&key].as_slice())
    }

    pub(super) fn avoidance(
        &mut self,
        radius: f64,
        layer: usize,
    ) -> Result<&[ExPolygon], ClipperError> {
        let radius = self.ceil_radius(radius);
        let radius_key = radius_key(radius);
        let key = (radius_key, layer);
        if !self.avoidance_cache.contains_key(&key) {
            // Native recursion unwinds from the nearest cached lower layer.
            let mut first = layer;
            while first > 0 && !self.avoidance_cache.contains_key(&(radius_key, first - 1)) {
                first -= 1;
            }
            for current in first..=layer {
                self.calculate_avoidance(radius, current)?;
            }
        }
        Ok(self.avoidance_cache[&key].as_slice())
    }

    fn calculate_avoidance(&mut self, radius: f64, layer: usize) -> Result<(), ClipperError> {
        let radius_key = radius_key(radius);
        let mut area = if layer == 0 {
            Vec::new()
        } else {
            offset_expolygons(
                &self.avoidance_cache[&(radius_key, layer - 1)],
                self.scale.scaled_delta(-self.max_move_distances[layer - 1]) as f32,
                JoinType::Miter,
                3.0,
            )?
        };
        area.extend_from_slice(self.collision(radius, layer)?);
        let area = union_expolygons(&area, &[])?;
        self.avoidance_cache
            .entry((radius_key, layer))
            .or_insert(area);
        Ok(())
    }
}

fn radius_key(radius: f64) -> u64 {
    if radius == 0.0 { 0 } else { radius.to_bits() }
}
