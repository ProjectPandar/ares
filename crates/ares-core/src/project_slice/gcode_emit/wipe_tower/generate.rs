//! `WipeTower2::generate` and the five `save_on_last_wipe` planning simulations.
use super::{
    generator::Generator,
    plan::{length_to_volume, wipe_depth},
    result::ToolChangeResult,
};
use crate::geometry::ClipperError;

impl Generator<'_> {
    pub(super) fn generate(&mut self) -> Result<Vec<Vec<ToolChangeResult>>, ClipperError> {
        if self.plan.layers.is_empty() {
            return Ok(Vec::new());
        }
        self.plan.plan_tower();
        self.current_height = 0.0;
        for _ in 0..5 {
            self.save_on_last_wipe()?;
            self.plan.plan_tower();
            self.current_height = 0.0;
        }
        self.rib_length = self
            .rib_length
            .max((self.plan.depth * self.plan.depth + self.plan.width * self.plan.width).sqrt());
        self.rib_length += self.settings.process.print.wipe_tower_extra_rib_length.0 as f32;
        self.rib_length = self.rib_length.max(0.0);
        self.rib_width = self
            .rib_width
            .min(self.plan.depth.min(self.plan.width) / 2.0);
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
            result.push(results);
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

    fn save_on_last_wipe(&mut self) -> Result<(), ClipperError> {
        for index in 0..self.plan.layers.len() {
            self.layer_index = index;
            let layer = &self.plan.layers[index];
            self.set_layer(layer.z, layer.height);
            if self.plan.layers[index].changes.is_empty() {
                continue;
            }
            let finish_after = self.first_nonsoluble_change();
            if finish_after.is_none() {
                self.finish_layer()?;
            }
            for change_index in 0..self.plan.layers[index].changes.len() {
                let tool = self.plan.layers[index].changes[change_index].new_tool;
                self.tool_change(Some(tool));
                if finish_after == Some(change_index) {
                    let extrusion = self.finish_layer()?.extrusion_length();
                    let layer = &self.plan.layers[index];
                    let change = &layer.changes[change_index];
                    let saved =
                        length_to_volume(extrusion, self.plan.perimeter_width, layer.height);
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
                    let spacing = if self.plan.first_layer == Some(index) {
                        self.plan.extra_flow
                    } else {
                        self.plan.extra_spacing
                    };
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
        }
        Ok(())
    }
}
