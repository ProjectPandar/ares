//! `WipeTower.cpp:3864–4083`: consumed Type1 block/layer generation order.
use super::{generator::Generator, result::Result};
use crate::geometry::ClipperError;
use std::collections::BTreeSet;

fn finish_remaining(
    generator: &mut Generator<'_>,
    wall_tool: usize,
    solid_blocks: &BTreeSet<usize>,
    fill: bool,
    output: (&mut Vec<Result>, &mut Option<Result>),
) -> std::result::Result<BTreeSet<usize>, ClipperError> {
    let (results, finish) = output;
    let index = generator.index;
    let mut next_solid = BTreeSet::new();
    for block_index in 0..generator.plan.blocks.len() {
        let block = &mut generator.plan.blocks[block_index];
        block.finish_depth[index] = block.start + block.depth;
        if f64::from(block.current) + 1e-4
            >= f64::from(block.start + block.layer_depths[index] - generator.plan.perimeter)
        {
            continue;
        }
        let mut tool = block.last_filament.or(block.last_nozzle);
        if let Some(first) = generator.plan.layers[index].changes.first()
            && generator.plan.categories[first.old] == block.category
            && tool.is_none()
        {
            tool = Some(first.old);
        }
        let tool = tool.unwrap_or(wall_tool);
        let different =
            block.solid_infill[index] && block.category != generator.plan.categories[tool];
        let interface = solid_blocks.contains(&block_index);
        let last_filament = block.last_filament;
        let result = if interface || different {
            if different {
                next_solid.insert(block_index);
            }
            generator.finish_block_solid(block_index, tool, interface && !different)?
        } else {
            let result = generator.finish_block(block_index, tool, fill)?;
            generator.plan.blocks[block_index].finish_depth[index] =
                generator.plan.blocks[block_index].current;
            result
        };
        if let Some(position) = results.iter().position(|r| r.common.new_tool == tool) {
            let previous = results.remove(position);
            results.insert(position, previous.merge(result, generator.max_speed()));
        } else if let Some(position) = last_filament
            .is_none()
            .then(|| results.iter().position(|r| r.common.initial_tool == tool))
            .flatten()
        {
            let next = results.remove(position);
            results.insert(position, result.merge(next, generator.max_speed()));
        } else if !result.common.gcode.is_empty() {
            *finish = Some(match finish.take() {
                Some(first) => first.merge(result, generator.max_speed()),
                None => result,
            });
        }
    }
    Ok(next_solid)
}

impl Generator<'_> {
    pub(super) fn generate(&mut self) -> std::result::Result<Vec<Vec<Result>>, ClipperError> {
        if self.plan.layers.is_empty() {
            return Ok(Vec::new());
        }
        if super::super::sparse_combination::enabled(self.settings) {
            let combined = super::super::sparse_combination::combine(
                self.settings,
                self.plan.layers.iter().map(|layer| {
                    (
                        layer.height,
                        layer
                            .changes
                            .first()
                            .zip(layer.changes.last())
                            .map(|(first, last)| (first.old, last.new)),
                    )
                }),
                self.plan
                    .first_layer
                    .expect("nonempty tower plan has a first layer"),
                self.current,
            );
            for (layer, (height, away)) in self.plan.layers.iter_mut().zip(combined) {
                layer.height = height;
                layer.combined_away = away;
            }
        }
        self.plan.plan_tower_new();
        if let Some(change) = self
            .plan
            .layers
            .iter()
            .find_map(|layer| layer.changes.first())
        {
            self.current = change.old;
        }
        self.used.fill(0.0);
        let wall = self.plan.wall_filament(self.current);
        let mut solid_blocks = BTreeSet::new();
        let mut output = Vec::new();
        self.outer_walls = vec![Vec::new(); self.plan.layers.len()];
        for index in 0..self.plan.layers.len() {
            if let Some(results) = self.generate_layer(index, wall, &mut solid_blocks)? {
                output.push(results);
            }
        }
        Ok(output)
    }

    fn generate_layer(
        &mut self,
        index: usize,
        wall: usize,
        solid_blocks: &mut BTreeSet<usize>,
    ) -> std::result::Result<Option<Vec<Result>>, ClipperError> {
        self.index = index;
        for block in &mut self.plan.blocks {
            block.current = block.start;
            block.last_filament = None;
            block.last_nozzle = None;
        }
        self.z = self.plan.layers[index].z;
        self.height = self.plan.layers[index].height;
        self.flow = self.flow_at(self.height);
        self.finished = false;
        let layer = &self.plan.layers[index];
        if layer.depth < self.plan.perimeter {
            return Ok(None);
        }
        if self.plan.blocks.len() == 1 && layer.depth < self.plan.depth - self.plan.perimeter {
            let step = self.plan.spacing * self.plan.perimeter;
            self.y_shift = ((self.plan.depth - layer.depth) / 2.0 / step).round() * step;
        }
        super::wall_gaps::prepare(self);
        let mut wall_tool = self
            .plan
            .layer_wall_filament(&self.plan.layers[index], wall);
        let only_wall = self.plan.smooth || (self.plan.wrapping && self.used_tools.len() <= 1);
        let fill = self.plan.layers[index].extruder_fill;
        let mut finish = None;
        let mut timelapse_wall = None;
        if wall_tool.is_none()
            && self
                .plan
                .blocks
                .iter()
                .any(|b| b.solid_infill[index] && b.category != self.plan.categories[self.current])
        {
            wall_tool = Some(self.current);
        }
        if wall_tool.is_none() {
            if only_wall {
                timelapse_wall = Some(self.only_outer_wall()?);
            }
            finish = Some(self.finish_layer(!only_wall, fill, true)?);
            for block in &mut self.plan.blocks {
                block.finish_depth[index] = block.start;
            }
        }
        let mut finish_index = None;
        if wall_tool.is_some() && only_wall {
            timelapse_wall = Some(self.only_outer_wall()?);
        }
        let mut results = Vec::<Result>::new();
        for i in 0..self.plan.layers[index].changes.len() {
            let change = &self.plan.layers[index].changes[i];
            let (old, new) = (change.old, change.new);
            let before = i == 0 && Some(old) == wall_tool;
            if before {
                finish = Some(self.finish_layer(!only_wall, false, false)?);
            }
            let new_block = self
                .plan
                .blocks
                .iter()
                .position(|b| b.category == self.plan.categories[new]);
            let old_block = self
                .plan
                .blocks
                .iter()
                .position(|b| b.category == self.plan.categories[old]);
            results.push(self.tool_change(
                new,
                new_block.is_some_and(|b| solid_blocks.contains(&b)),
                old_block.is_some_and(|b| solid_blocks.contains(&b)),
            )?);
            if !before && Some(new) == wall_tool {
                finish = Some(self.finish_layer(!only_wall, false, false)?);
                finish_index = Some(i);
            }
        }
        let mut next_solid = BTreeSet::new();
        if let Some(wall_tool) = wall_tool {
            if self.plan.layers[index].changes.is_empty() {
                finish = Some(self.finish_layer(!only_wall, false, false)?);
            }
            next_solid = finish_remaining(
                self,
                wall_tool,
                solid_blocks,
                fill,
                (&mut results, &mut finish),
            )?;
        }
        *solid_blocks = next_solid;
        if results.is_empty() {
            results.push(finish.expect("no-change tower layer retains finish output"));
        } else if let Some(finish) = finish {
            if let Some(position) = finish_index {
                let previous = results.remove(position);
                results.insert(position, previous.merge(finish, self.max_speed()));
            } else {
                let next = results.remove(0);
                results.insert(0, finish.merge(next, self.max_speed()));
            }
        }
        if let Some(wall) = timelapse_wall
            && !wall.common.gcode.is_empty()
        {
            results.insert(0, wall);
        }
        if self.plan.layers[index].combined_away {
            for result in &mut results {
                result.common.combined_away = true;
            }
        }
        Ok(Some(results))
    }
}
