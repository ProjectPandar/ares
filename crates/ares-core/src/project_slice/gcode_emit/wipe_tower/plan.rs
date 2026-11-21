//! `WipeTowerInfo`, `plan_toolchange` and downward `plan_tower` depth propagation.
use super::{filament::Filament, generator::use_gap_wall};
use crate::ProjectSettings;

pub(super) struct ToolChange {
    pub(super) old_tool: usize,
    pub(super) new_tool: usize,
    pub(super) required_depth: f32,
    pub(super) ramming_depth: f32,
    pub(super) first_wipe_line: f32,
    pub(super) wipe_volume: f32,
    pub(super) wipe_volume_total: f32,
}

pub(super) struct Layer {
    pub(super) z: f32,
    pub(super) height: f32,
    pub(super) depth: f32,
    pub(super) changes: Vec<ToolChange>,
}
impl Layer {
    pub(super) fn changes_depth(&self) -> f32 {
        self.changes.iter().map(|c| c.required_depth).sum()
    }
}

pub(super) struct Plan {
    pub(super) layers: Vec<Layer>,
    pub(super) first_layer: Option<usize>,
    pub(super) width: f32,
    pub(super) depth: f32,
    pub(super) height: f32,
    pub(super) perimeter_width: f32,
    pub(super) extra_flow: f32,
    pub(super) extra_spacing: f32,
    pub(super) ramming_spacing: f32,
    pub(super) ramming: bool,
    semm: bool,
    use_gap_wall: bool,
    pub(super) no_sparse: bool,
}

impl Plan {
    pub(super) fn new(settings: &ProjectSettings, filaments: &[Filament]) -> Self {
        let print = &settings.process.print;
        Self {
            layers: Vec::new(),
            first_layer: None,
            width: print.prime_tower_width.0 as f32,
            depth: 0.0,
            height: 0.0,
            perimeter_width: filaments.last().unwrap().nozzle * 1.25,
            extra_flow: (print.wipe_tower_extra_flow.0 / 100.0) as f32,
            extra_spacing: (print.wipe_tower_extra_spacing.0 / 100.0
                * print.wipe_tower_extra_flow.0
                / 100.0) as f32,
            ramming_spacing: (print.wipe_tower_extra_spacing.0 / 100.0) as f32,
            ramming: settings.printer.gcode.enable_filament_ramming.0,
            semm: settings.printer.gcode.single_extruder_multi_material.0,
            use_gap_wall: use_gap_wall(print),
            no_sparse: super::sparse_layers_skipped(settings),
        }
    }

    pub(super) fn add(
        &mut self,
        (z, height): (f32, f32),
        (old_tool, new_tool): (usize, usize),
        volume: f32,
        filaments: &[Filament],
    ) {
        if self.layers.last().is_none_or(|layer| layer.z + 1e-3 < z) {
            self.layers.push(Layer {
                z,
                height,
                depth: 0.0,
                changes: Vec::new(),
            });
        }
        let layer_index = self.layers.len() - 1;
        if self.first_layer.is_none()
            && (!self.no_sparse || old_tool != new_tool || self.layers.len() == 1)
        {
            self.first_layer = Some(layer_index);
        }
        if old_tool == new_tool {
            return;
        }
        let first_layer_plan = self.first_layer == Some(layer_index);
        let change = self.set_toolchange(
            (old_tool, new_tool),
            (height, volume),
            first_layer_plan,
            filaments,
        );
        self.layers[layer_index].changes.push(change);
    }

    /// The rib-square replan of every toolchange at its layer height, with the reduced wipe
    /// volume (`WipeTower2.cpp:2495-2499`).
    pub(super) fn replan_toolchanges(&mut self, filaments: &[Filament]) {
        for index in 0..self.layers.len() {
            let layer = &self.layers[index];
            let first_layer_plan = self.first_layer == Some(index);
            let changes = layer
                .changes
                .iter()
                .map(|change| {
                    self.set_toolchange(
                        (change.old_tool, change.new_tool),
                        (layer.height, change.wipe_volume),
                        first_layer_plan,
                        filaments,
                    )
                })
                .collect();
            self.layers[index].changes = changes;
        }
    }

    /// `WipeTower2::set_toolchange` (`WipeTower2.cpp:2275`): the depth one toolchange reserves.
    pub(super) fn set_toolchange(
        &self,
        (old_tool, new_tool): (usize, usize),
        (height, volume): (f32, f32),
        first_layer_plan: bool,
        filaments: &[Filament],
    ) -> ToolChange {
        let filament = &filaments[old_tool];
        let width = self.width - 3.0 * self.perimeter_width;
        let ramming_step_time = if self.semm {
            0.25
        } else {
            filament.multitool_ramming_time
        };
        let length = volume_to_length(
            ramming_step_time * filament.ramming_speeds.iter().sum::<f32>(),
            self.perimeter_width * filament.ramming_width,
            height,
        );
        let do_ramming = (self.semm && self.ramming) || filament.multitool_ramming;
        let ramming_depth = if do_ramming {
            ((length / width) as i32 + 1) as f32
                * (self.perimeter_width * filament.ramming_width * filament.ramming_step)
                * self.ramming_spacing
        } else {
            0.0
        };
        let boundary_wipe_start = do_ramming && !self.semm && self.use_gap_wall;
        let first_wipe_line = if do_ramming && !boundary_wipe_start {
            -(width * (length / width - (length / width) as i32 as f32) - width)
        } else {
            0.0
        };
        let first_volume = length_to_volume(
            first_wipe_line,
            self.perimeter_width * self.extra_flow,
            height,
        );
        let spacing = if first_layer_plan {
            self.extra_flow
        } else {
            self.extra_spacing
        };
        let wiping_depth = wipe_depth(
            volume - first_volume,
            (self.perimeter_width, height),
            self.extra_flow,
            spacing,
            width,
        );
        ToolChange {
            old_tool,
            new_tool,
            required_depth: ramming_depth + wiping_depth,
            ramming_depth,
            first_wipe_line,
            wipe_volume: volume,
            wipe_volume_total: volume,
        }
    }

    pub(super) fn plan_tower(&mut self) {
        self.depth = 0.0;
        self.height = self.layers.last().map_or(0.0, |layer| layer.z);
        for layer in &mut self.layers {
            layer.depth = 0.0;
        }
        for index in (0..self.layers.len()).rev() {
            let depth = self.layers[index]
                .depth
                .max(self.layers[index].changes_depth());
            self.layers[index].depth = depth;
            if depth > self.depth - self.perimeter_width {
                self.depth = depth + self.perimeter_width;
            }
            let transition_gap = 2.0 * self.perimeter_width;
            for lower in self.layers[..index]
                .iter_mut()
                .rev()
                .filter(|lower| lower.depth - depth < transition_gap)
            {
                lower.depth = depth;
            }
        }
    }
}

pub(super) fn volume_to_length(volume: f32, width: f32, height: f32) -> f32 {
    (volume / (height * (width - height * (1.0 - std::f32::consts::PI / 4.0)))).max(0.0)
}
pub(super) fn length_to_volume(length: f32, width: f32, height: f32) -> f32 {
    (length * height * (width - height * (1.0 - std::f32::consts::PI / 4.0))).max(0.0)
}
pub(super) fn wipe_depth(
    volume: f32,
    (width, height): (f32, f32),
    flow: f32,
    spacing: f32,
    box_width: f32,
) -> f32 {
    let length = (volume_to_length(volume, width, height) / flow).max(0.0);
    ((length / box_width) as i32 + 1) as f32 * width * spacing
}
