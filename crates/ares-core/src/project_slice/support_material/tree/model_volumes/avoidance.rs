//! `TreeModelVolumes::getAvoidance/calculateAvoidance`, per-radius task owner.

mod movement;
mod propagation;

use super::{TreeModelVolumes, cache::RadiusLayerPolygonCache};
use crate::geometry::{ClipperError, Coord, Polygon};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(in super::super) enum AvoidanceType {
    Slow,
    FastSafe,
    Fast,
}

impl AvoidanceType {
    fn index(self) -> usize {
        match self {
            Self::Slow => 0,
            Self::FastSafe => 1,
            Self::Fast => 2,
        }
    }
}

impl TreeModelVolumes<'_> {
    pub(in super::super) fn avoidance(
        &mut self,
        key: (Coord, usize),
        kind: AvoidanceType,
        to_model: bool,
        minimum_xy: bool,
    ) -> Result<&[Polygon], ClipperError> {
        let (original_radius, layer) = key;
        if layer == 0 {
            return self.collision(original_radius, layer, minimum_xy);
        }
        let radius = self.radii.ceil(original_radius, minimum_xy);
        let kind = if kind == AvoidanceType::FastSafe && radius >= self.holefree_limit() {
            AvoidanceType::Fast
        } else {
            kind
        };
        if self
            .avoidance_cache(kind, to_model)
            .area(radius, layer)
            .is_none()
        {
            self.calculate_avoidance_batch(&[(radius, layer)], !to_model, to_model)?;
        }
        Ok(self
            .avoidance_cache(kind, to_model)
            .area(radius, layer)
            .expect("avoidance calculation inserted every requested layer"))
    }

    pub(super) fn holefree_limit(&self) -> Coord {
        self.settings.increase_radius_until_radius + self.settings.xy_distance
            - self.settings.xy_min_distance
    }

    fn avoidance_cache(&self, kind: AvoidanceType, to_model: bool) -> &RadiusLayerPolygonCache {
        &self.avoidance_caches[usize::from(to_model)][kind.index()]
    }

    pub(super) fn calculate_avoidance_batch(
        &mut self,
        keys: &[(Coord, usize)],
        to_build_plate: bool,
        to_model: bool,
    ) -> Result<(), ClipperError> {
        let mut tasks = Vec::with_capacity(
            (usize::from(to_build_plate) + usize::from(to_model)) * keys.len() * 3,
        );
        for index in 0..keys.len() * 6 {
            let kind = [
                AvoidanceType::Slow,
                AvoidanceType::FastSafe,
                AvoidanceType::Fast,
            ][index % 3];
            let (radius, max_layer) = keys[index / 6];
            let destination = (index / 3) & 1 != 0;
            let begin = self
                .avoidance_cache(kind, destination)
                .max_calculated_layer(radius)
                .map_or(1, |layer| layer + 1);
            if begin <= max_layer
                && (if destination {
                    to_model
                } else {
                    to_build_plate
                })
                && (kind != AvoidanceType::FastSafe || radius < self.holefree_limit())
            {
                tasks.push((radius, max_layer, kind, destination, begin));
            }
        }
        for (radius, max_layer, kind, destination, begin) in tasks {
            self.propagate_avoidance(radius, begin..max_layer + 1, kind, destination)?;
        }
        Ok(())
    }
}
