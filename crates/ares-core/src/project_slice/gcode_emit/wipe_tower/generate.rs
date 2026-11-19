//! `WipeTower2::generate` and the five `save_on_last_wipe` planning simulations.
use super::{
    estimate::{EPSILON, align_ceil, limit_depth_by_height},
    generator::Generator,
    plan::{length_to_volume, wipe_depth},
    result::ToolChangeResult,
};
use crate::{ProcessWipeTowerWallType, geometry::ClipperError};

impl Generator<'_> {
    pub(super) fn generate(&mut self) -> Result<Vec<Vec<ToolChangeResult>>, ClipperError> {
        if self.plan.layers.is_empty() {
            return Ok(Vec::new());
        }
        if super::sparse_combination::enabled(self.settings) {
            let combined = super::sparse_combination::combine(
                self.settings,
                self.plan.layers.iter().map(|layer| {
                    (
                        layer.height,
                        layer
                            .changes
                            .first()
                            .zip(layer.changes.last())
                            .map(|(first, last)| (first.old_tool, last.new_tool)),
                    )
                }),
                self.plan
                    .first_layer
                    .expect("nonempty tower plan has a first layer"),
                self.current_tool,
            );
            for (layer, (height, away)) in self.plan.layers.iter_mut().zip(combined) {
                layer.height = height;
                layer.combined_away = away;
            }
        }
        self.plan.plan_tower();
        self.current_height = 0.0;
        for _ in 0..5 {
            self.save_on_last_wipe()?;
            self.plan.plan_tower();
            self.current_height = 0.0;
        }
        if self.settings.process.print.wipe_tower_wall_type == ProcessWipeTowerWallType::Rib {
            self.square_rib_tower();
        }
        // `WipeTower2.cpp:2509-2514`: a negative extra length cannot shrink the ribs below the
        // diagonal.
        let diagonal =
            (self.plan.depth * self.plan.depth + self.plan.width * self.plan.width).sqrt();
        self.rib_length = self.rib_length.max(diagonal);
        self.rib_length += self.settings.process.print.wipe_tower_extra_rib_length.0 as f32;
        self.rib_length = diagonal.max(self.rib_length);
        self.rib_width = self
            .rib_width
            .min(self.plan.depth.min(self.plan.width) / 2.0);
        if self.use_gap_wall() {
            self.compute_wall_skip_points();
        }
        self.layer_index = 0;
        self.current_height = 0.0;
        if let Some(change) = self
            .plan
            .layers
            .iter()
            .find_map(|layer| layer.changes.first())
        {
            self.current_tool = change.old_tool;
        }
        self.used_filament.fill(0.0);
        self.used_until_layer
            .push((0.0, self.used_filament.clone()));
        self.old_temperature = -1;
        let mut result = Vec::with_capacity(self.plan.layers.len());
        for layer_index in 0..self.plan.layers.len() {
            let layer = &self.plan.layers[layer_index];
            self.set_layer(layer.z, layer.height);
            self.rotation += 180.0;
            let layer = &self.plan.layers[layer_index];
            if layer.depth < self.plan.depth - self.plan.perimeter_width {
                self.y_shift = (self.plan.depth - layer.depth - self.plan.perimeter_width) / 2.0;
            }
            result.push(self.generate_layer_changes(layer_index)?);
            if self.used_until_layer.last().unwrap().0 != self.z {
                self.used_until_layer
                    .push((self.z, self.used_filament.clone()));
            } else {
                self.used_until_layer
                    .last_mut()
                    .unwrap()
                    .1
                    .clone_from(&self.used_filament);
            }
        }
        Ok(result)
    }

    /// `WipeTower2::compute_wall_skip_points` (`WipeTower2.cpp:2453-2463`): a wall opening at
    /// each toolchange's entry.
    fn compute_wall_skip_points(&mut self) {
        self.wall_skip_points = self
            .plan
            .layers
            .iter()
            .map(|layer| {
                let mut depth_traversed = 0.0;
                layer
                    .changes
                    .iter()
                    .map(|change| {
                        let entry = self.toolchange_entry_pos(depth_traversed);
                        depth_traversed += change.required_depth;
                        entry
                    })
                    .collect()
            })
            .collect();
    }

    /// `WipeTower2.cpp:2485-2507`: a rib tower prints square, keeping the planned purge area,
    /// and grows its ribs when it is shallower than the height-based minimum.
    fn square_rib_tower(&mut self) {
        let max_depth = self
            .plan
            .layers
            .iter()
            .fold(0.0_f32, |max_depth, layer| max_depth.max(layer.depth));
        if f64::from(max_depth) > EPSILON {
            self.plan.width = align_ceil(
                (max_depth * self.plan.width).sqrt(),
                self.plan.perimeter_width,
            );
            self.plan.replan_toolchanges(&self.filaments);
            self.plan.plan_tower();
        }
        let min_depth = limit_depth_by_height(self.plan.height);
        if f64::from(self.plan.depth) + EPSILON < f64::from(min_depth) {
            self.rib_length = self.rib_length.max(min_depth * 2.0_f32.sqrt());
        }
    }

    fn save_on_last_wipe(&mut self) -> Result<(), ClipperError> {
        for index in 0..self.plan.layers.len() {
            self.save_layer_on_last_wipe(index)?;
        }
        Ok(())
    }

    fn generate_layer_changes(
        &mut self,
        layer_index: usize,
    ) -> Result<Vec<ToolChangeResult>, ClipperError> {
        let finish_after = self.first_nonsoluble_change();
        let mut finish = if finish_after.is_none() {
            Some(self.finish_layer()?)
        } else {
            None
        };
        let mut results = Vec::with_capacity(self.plan.layers[layer_index].changes.len());
        for index in 0..self.plan.layers[layer_index].changes.len() {
            let tool = self.plan.layers[layer_index].changes[index].new_tool;
            results.push(self.tool_change(Some(tool)));
            if finish_after == Some(index) {
                finish = Some(self.finish_layer()?);
            }
        }
        let finish = finish.unwrap();
        if results.is_empty() {
            results.push(finish);
        } else if let Some(index) = finish_after {
            let original = results.remove(index);
            results.insert(index, original.merge(finish));
        } else {
            let first = results.remove(0);
            let mut merged = finish.merge(first);
            merged.force_travel = true;
            results.insert(0, merged);
        }
        if self.plan.layers[layer_index].combined_away {
            for result in &mut results {
                result.combined_away = true;
            }
        }
        Ok(results)
    }

    fn save_layer_on_last_wipe(&mut self, index: usize) -> Result<(), ClipperError> {
        self.layer_index = index;
        let layer = &self.plan.layers[index];
        self.set_layer(layer.z, layer.height);
        if self.plan.layers[index].changes.is_empty() {
            return Ok(());
        }
        let finish_after = self.first_nonsoluble_change();
        if finish_after.is_none() {
            self.finish_layer()?;
        }
        let spacing = if self.plan.first_layer == Some(index) {
            self.plan.extra_flow
        } else {
            self.plan.extra_spacing
        };
        for change_index in 0..self.plan.layers[index].changes.len() {
            let tool = self.plan.layers[index].changes[change_index].new_tool;
            self.tool_change(Some(tool));
            let saved = if finish_after == Some(change_index) {
                let extrusion = self.finish_layer()?.extrusion_length();
                Some(length_to_volume(
                    extrusion,
                    self.plan.perimeter_width,
                    self.plan.layers[index].height,
                ))
            } else if self.plan.layers[index].changes[change_index].wipe_volume
                < self.filaments[tool].minimal_purge
            {
                // WipeTower2.cpp:2381-2386: non-finishing changes save no volume.
                Some(0.0)
            } else {
                None
            };
            if let Some(saved) = saved {
                let layer = &self.plan.layers[index];
                let change = &layer.changes[change_index];
                let remaining = self.filaments[tool]
                    .minimal_purge
                    .max(change.wipe_volume_total - saved);
                let volume = (remaining
                    - length_to_volume(
                        change.first_wipe_line,
                        self.plan.perimeter_width * self.plan.extra_flow,
                        layer.height,
                    ))
                .max(0.0);
                let depth = wipe_depth(
                    volume,
                    (self.plan.perimeter_width, layer.height),
                    self.plan.extra_flow,
                    spacing,
                    self.plan.width - 3.0 * self.plan.perimeter_width,
                );
                let change = &mut self.plan.layers[index].changes[change_index];
                change.required_depth = change.ramming_depth + depth;
                change.wipe_volume = remaining;
            }
        }
        Ok(())
    }
}
