//! `TreeModelVolumes.hpp::RadiusLayerPolygonCache`.

use std::collections::BTreeMap;

use crate::geometry::{Coord, Polygon};

#[derive(Default)]
pub(super) struct RadiusLayerPolygonCache {
    layers: Vec<BTreeMap<Coord, Vec<Polygon>>>,
}

impl RadiusLayerPolygonCache {
    pub(super) fn clear_all_but_radius_zero(&mut self) {
        for layer in &mut self.layers {
            // Source retains the first radius entry, even if it is not zero.
            let first = layer.keys().next().copied();
            layer.retain(|&radius, _| Some(radius) == first);
        }
    }

    pub(super) fn insert(&mut self, radius: Coord, begin: usize, polygons: Vec<Vec<Polygon>>) {
        self.layers
            .resize_with(self.layers.len().max(begin + polygons.len()), BTreeMap::new);
        for (layer, polygons) in polygons.into_iter().enumerate() {
            self.layers[begin + layer].entry(radius).or_insert(polygons);
        }
    }

    pub(super) fn area(&self, radius: Coord, layer: usize) -> Option<&[Polygon]> {
        self.layers.get(layer)?.get(&radius).map(Vec::as_slice)
    }

    pub(super) fn lower_bound_area(
        &self,
        radius: Coord,
        layer: usize,
    ) -> Option<(Coord, &[Polygon])> {
        self.layers
            .get(layer)?
            .range(..=radius)
            .next_back()
            .map(|(&radius, polygons)| (radius, polygons.as_slice()))
    }

    pub(super) fn max_calculated_layer(&self, radius: Coord) -> Option<usize> {
        // Upstream returns -1 even when layer zero alone contains this radius.
        self.layers
            .iter()
            .enumerate()
            .skip(1)
            .rev()
            .find_map(|(layer, data)| data.contains_key(&radius).then_some(layer))
    }
}
