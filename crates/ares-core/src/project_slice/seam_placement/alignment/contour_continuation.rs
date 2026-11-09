// OrcaSlicer GCode/SeamContour.hpp and SeamContour.cpp::Resolver.
mod geometry;
mod index;
mod selection;

use std::collections::BTreeMap;

use index::Index;

pub(super) struct Contour {
    pub(super) points: Vec<[f64; 2]>,
    pub(super) hole: bool,
    pub(super) reliable: bool,
    pub(super) width: f64,
    pub(super) nozzle: f64,
    pub(super) min: [f64; 2],
    pub(super) max: [f64; 2],
}

impl Contour {
    fn update_bounds(&mut self) {
        self.min = [f64::INFINITY; 2];
        self.max = [f64::NEG_INFINITY; 2];
        for point in &self.points {
            for (axis, &coordinate) in point.iter().enumerate() {
                self.min[axis] = self.min[axis].min(coordinate);
                self.max[axis] = self.max[axis].max(coordinate);
            }
        }
    }
}

pub(super) type Layers = Vec<Vec<Contour>>;

pub(super) struct Options {
    bounds_nozzle_factor: f64,
    radius_width_factor: f64,
    noise_width_factor: f64,
    normal_cosine: f64,
    dominance_width_factor: f64,
    dominance_angle_degrees: f64,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            bounds_nozzle_factor: 1.25,
            radius_width_factor: 4.0,
            noise_width_factor: 0.125,
            normal_cosine: 0.5,
            dominance_width_factor: 0.25,
            dominance_angle_degrees: 5.0,
        }
    }
}

// Lossless variants of Outcome, Method and uncertain; only Local can be
// None/Multiple/uncertain. An empty restriction leaves the KD search unchanged.
#[derive(Clone)]
pub(super) enum Result {
    UniqueBounds(usize),
    UniqueLocal(usize),
    Multiple(Vec<usize>),
    None,
    Uncertain,
}

impl Result {
    pub(super) fn contours(&self) -> &[usize] {
        match self {
            Self::UniqueBounds(index) | Self::UniqueLocal(index) => std::slice::from_ref(index),
            Self::Multiple(indices) => indices,
            Self::None | Self::Uncertain => &[],
        }
    }
}

type Key = (usize, usize, usize, isize, u64);

pub(super) struct Resolver {
    layers: Layers,
    indices: Vec<Index>,
    options: Options,
    cache: BTreeMap<Key, Result>,
}

impl Resolver {
    pub(super) fn new(mut layers: Layers, options: Options) -> Self {
        let indices = layers
            .iter_mut()
            .map(|layer| {
                for contour in &mut *layer {
                    contour.update_bounds();
                }
                Index::new(layer)
            })
            .collect();
        Self {
            layers,
            indices,
            options,
            cache: BTreeMap::new(),
        }
    }

    pub(super) fn resolve(
        &mut self,
        (layer, contour, vertex): (usize, usize, usize),
        direction: isize,
        mut radius: f64,
    ) -> Result {
        if direction != -1 && direction != 1 {
            return Result::None;
        }
        let source = &self.layers[layer][contour];
        if !source.reliable {
            return Result::Uncertain;
        }
        if radius < 0.0 {
            radius = source.width * self.options.radius_width_factor;
        }
        if !radius.is_finite() {
            return Result::None;
        }
        // Native's double-key comparison treats both signed zeros as equal.
        let radius_key = if radius == 0.0 { 0 } else { radius.to_bits() };
        let key = (layer, contour, vertex, direction, radius_key);
        if let Some(cached) = self.cache.get(&key) {
            return cached.clone();
        }
        let target = layer as isize + direction;
        let result =
            if target >= 0 && (target as usize) < self.layers.len() && source.points.len() >= 3 {
                self.select(
                    (layer, contour),
                    source.points[vertex],
                    (direction, radius),
                    true,
                )
            } else {
                Result::None
            };
        self.cache.insert(key, result.clone());
        result
    }
}
