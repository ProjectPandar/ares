//! `WipeTower.cpp:3516–3703`: category blocks and downward depth propagation.
use super::{
    category_depths::Depths,
    plan::{Layer, Plan},
};

fn category_depths(layer: &Layer, nozzles: &[usize], ids: &[i32]) -> Depths {
    let mut categories = Depths::default();
    for change in &layer.changes {
        if nozzles[change.old] == nozzles[change.new] {
            categories.add(ids[change.new], change.depth);
        } else {
            categories.add(ids[change.old], change.nozzle_depth);
            categories.add(ids[change.new], change.depth - change.nozzle_depth);
        }
    }
    categories
}

fn install_layer(blocks: &mut Vec<Block>, layer: usize, categories: &Depths, count: usize) {
    for (category, depth) in categories.iter() {
        let index = blocks
            .iter()
            .position(|b| b.category == category)
            .unwrap_or_else(|| {
                let index = blocks.len();
                blocks.push(Block {
                    category,
                    layer_depths: vec![0.0; count],
                    solid_infill: vec![false; count],
                    finish_depth: vec![0.0; count],
                    depth: 0.0,
                    start: 0.0,
                    current: 0.0,
                    last_filament: None,
                    last_nozzle: None,
                });
                index
            });
        blocks[index].depth = blocks[index].depth.max(depth);
        blocks[index].layer_depths[layer] = depth;
    }
}

fn mark_solid(block: &mut Block, depths: &[Depths]) {
    for (layer, depth) in depths.iter().enumerate() {
        if depth.get(block.category).abs() >= 1e-4 {
            continue;
        }
        block.solid_infill[layer] = (1..=4).rev().any(|offset| {
            depths
                .get(layer + offset)
                .is_some_and(|upper| upper.get(block.category).abs() >= 1e-4)
        });
    }
}

fn framework(blocks: &mut [Block], layers: &mut [Layer]) {
    for (index, layer) in layers.iter_mut().enumerate().skip(1) {
        layer.depth = 0.0;
        for block in blocks.iter_mut() {
            block.layer_depths[index] = block.layer_depths[0];
            layer.depth += block.layer_depths[index];
        }
    }
}

pub(super) struct Block {
    pub(super) category: i32,
    pub(super) layer_depths: Vec<f32>,
    pub(super) solid_infill: Vec<bool>,
    pub(super) finish_depth: Vec<f32>,
    pub(super) depth: f32,
    pub(super) start: f32,
    pub(super) current: f32,
    pub(super) last_filament: Option<usize>,
    pub(super) last_nozzle: Option<usize>,
}

impl Plan {
    pub(super) fn generate_blocks(&mut self) {
        let depths = self
            .layers
            .iter()
            .map(|layer| category_depths(layer, &self.nozzle_map, &self.categories))
            .collect::<Vec<_>>();
        self.blocks.clear();
        for (layer, categories) in depths.iter().enumerate() {
            install_layer(&mut self.blocks, layer, categories, self.layers.len());
        }
        for block in &mut self.blocks {
            mark_solid(block, &depths);
        }
        for layer in (0..self.layers.len()).rev() {
            self.layers[layer].depth = 0.0;
            for block in &mut self.blocks {
                let upper = block
                    .layer_depths
                    .get(layer + 1)
                    .copied()
                    .unwrap_or(block.layer_depths[layer]);
                block.layer_depths[layer] = block.layer_depths[layer].max(upper);
                self.layers[layer].depth += block.layer_depths[layer];
            }
        }
        if self.framework {
            framework(&mut self.blocks, &mut self.layers);
        }
    }

    pub(super) fn update_depth(&mut self, minimum: f32) {
        self.depth = 0.0;
        let mut start = self.perimeter;
        for block in &mut self.blocks {
            block.depth *= self.spacing;
            block.start = start;
            start += block.depth;
            self.depth += block.depth;
            for depth in &mut block.layer_depths {
                *depth *= self.spacing;
            }
            // Source multiplies every plan depth once per block, not once globally.
            for layer in &mut self.layers {
                layer.depth *= self.spacing;
            }
        }
        if self.depth > 0.0 {
            self.depth += self.perimeter;
        }
        if self.wrapping || self.smooth {
            if self.depth.abs() < 1e-4 {
                self.depth = minimum;
            }
            for layer in &mut self.layers {
                layer.depth = self.depth;
            }
        }
    }
}
