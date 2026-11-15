//! `WipeTower.hpp::WipeTowerInfo`; `WipeTower.cpp:2449–2510,3705–3806`.
use super::blocks::Block;
use crate::ProjectSettings;

pub(super) struct Change {
    pub(super) old: usize,
    pub(super) new: usize,
    pub(super) depth: f32,
    pub(super) nozzle_depth: f32,
    pub(super) volume: f32,
    pub(super) length: f32,
    pub(super) purge: f32,
}

pub(super) struct Layer {
    pub(super) z: f32,
    pub(super) height: f32,
    pub(super) depth: f32,
    pub(super) spacing: f32,
    pub(super) extruder_fill: bool,
    pub(super) changes: Vec<Change>,
}

pub(super) struct Plan {
    pub(super) layers: Vec<Layer>,
    pub(super) blocks: Vec<Block>,
    pub(super) first_layer: Option<usize>,
    pub(super) width: f32,
    pub(super) depth: f32,
    pub(super) height: f32,
    pub(super) perimeter: f32,
    pub(super) spacing: f32,
    pub(super) infill_gap: f32,
    pub(super) brim: f32,
    pub(super) rib_length: f32,
    pub(super) rib_width: f32,
    pub(super) rib: bool,
    pub(super) smooth: bool,
    pub(super) wrapping: bool,
    pub(super) framework: bool,
    pub(super) nozzle_map: Vec<usize>,
    pub(super) categories: Vec<i32>,
    pub(super) change_lengths: Vec<f32>,
    pub(super) area: f32,
    pub(super) tpu: bool,
    no_sparse: bool,
    extra_rib: f32,
}

impl Plan {
    pub(super) fn new(settings: &ProjectSettings, used: &[usize]) -> Self {
        use super::super::filament::at;
        let p = &settings.process.print;
        let g = &settings.filament.gcode;
        let count = g.filament_colour.0.len();
        let diameter = g.filament_diameter.0[0].0;
        Self {
            layers: Vec::new(),
            blocks: Vec::new(),
            first_layer: None,
            width: p.prime_tower_width.0 as f32,
            depth: 0.0,
            height: 0.0,
            perimeter: at(&settings.project.print.nozzle_diameter.0, count - 1).0 as f32 * 1.25,
            spacing: p.prime_tower_infill_gap.0 as f32 / 100.0,
            infill_gap: 0.0,
            brim: p.prime_tower_brim_width.0 as f32,
            rib_length: 0.0,
            rib_width: p.wipe_tower_rib_width.0 as f32,
            rib: p.wipe_tower_wall_type == crate::ProcessWipeTowerWallType::Rib,
            smooth: p.timelapse_type == crate::ProcessTimelapseType::Smooth,
            wrapping: settings.process.gcode.enable_wrapping_detection.0,
            framework: p.prime_tower_enable_framework.0,
            nozzle_map: settings
                .project
                .gcode
                .filament_map
                .0
                .iter()
                .map(|id| (id.0 - 1) as usize)
                .collect(),
            categories: (0..count)
                .map(|i| at(&g.filament_adhesiveness_category.0, i).0)
                .collect(),
            change_lengths: (0..count)
                .map(|i| at(&g.filament_change_length.0, i).0 as f32)
                .collect(),
            area: (std::f64::consts::FRAC_PI_4 * diameter.powi(2)) as f32,
            tpu: used.iter().any(|&i| at(&g.filament_type.0, i) == "TPU"),
            no_sparse: settings.process.gcode.wipe_tower_no_sparse_layers.0,
            extra_rib: p.wipe_tower_extra_rib_length.0 as f32,
        }
    }

    pub(super) fn add(
        &mut self,
        position: (f32, f32),
        tools: (usize, usize),
        volume: f32,
        purge: f32,
    ) {
        let (z, height) = position;
        let (old, new) = tools;
        if self.layers.last().is_none_or(|layer| layer.z + 1e-3 < z) {
            self.layers.push(Layer {
                z,
                height,
                depth: 0.0,
                spacing: 1.0,
                extruder_fill: true,
                changes: Vec::new(),
            });
        }
        if self.first_layer.is_none() && (!self.no_sparse || old != new) {
            self.first_layer = Some(self.layers.len() - 1);
        }
        if old == new {
            return;
        }
        let width = self.width - 2.0 * self.perimeter;
        if f64::from(width) <= 1e-4 {
            return;
        }
        let length = super::super::plan::volume_to_length(volume, self.perimeter, height);
        let nozzle_depth = self.nozzle_depth(old, new, height);
        self.layers.last_mut().unwrap().changes.push(Change {
            old,
            new,
            depth: (length / width).ceil() * self.perimeter + nozzle_depth,
            nozzle_depth,
            volume,
            length,
            purge,
        });
    }

    fn nozzle_depth(&self, old: usize, new: usize, height: f32) -> f32 {
        if self.nozzle_map[old] == self.nozzle_map[new] {
            return 0.0;
        }
        let perimeter = 2.0 * self.perimeter;
        let flow = height * (perimeter - height * (1.0 - std::f32::consts::PI / 4.0)) / self.area;
        let length = f64::from(self.change_lengths[old]) / f64::from(flow);
        let lines = (length / f64::from(self.width - 2.0 * perimeter) + 1.0) as i32;
        if self.tpu {
            2.0 * lines as f32 * perimeter
        } else {
            lines as f32 * perimeter
        }
    }

    fn resize_rib_changes(&mut self) {
        for index in 0..self.layers.len() {
            for change_index in 0..self.layers[index].changes.len() {
                let change = &self.layers[index].changes[change_index];
                let nozzle_depth =
                    self.nozzle_depth(change.old, change.new, self.layers[index].height);
                let change = &mut self.layers[index].changes[change_index];
                change.nozzle_depth = nozzle_depth;
                change.depth = (change.length / (self.width - 2.0 * self.perimeter)).ceil()
                    * self.infill_gap
                    + nozzle_depth;
            }
        }
    }

    pub(super) fn calc_block_infill_gap(&mut self) {
        let width = self.width - 2.0 * self.perimeter;
        self.infill_gap = self.perimeter * self.spacing;
        let gap = self.infill_gap;
        for layer in &mut self.layers {
            for change in &mut layer.changes {
                change.depth = (change.length / width).ceil() * gap + change.nozzle_depth;
            }
        }
        self.spacing = 1.0;
    }

    pub(super) fn plan_tower_new(&mut self) {
        self.height = self.layers.last().map_or(0.0, |layer| layer.z);
        if self.brim < 0.0 {
            self.brim = if self.height < 100.0 {
                self.height / 100.0 * 8.0
            } else {
                8.0
            };
        }
        self.calc_block_infill_gap();
        if self.rib {
            self.generate_blocks();
            let depth = self.blocks.iter().map(|b| b.depth).sum::<f32>() + self.perimeter;
            self.width = (depth * self.width * self.spacing).sqrt();
            self.width = (self.width / self.perimeter).ceil() * self.perimeter;
            self.resize_rib_changes();
        }
        self.generate_blocks();
        let mut depth = self.blocks.iter().map(|b| b.depth).sum::<f32>();
        let minimum = minimum_depth(self.height);
        if self.wrapping && f64::from(depth) < 1e-4 {
            depth = 10.0;
            if self.rib {
                self.width = depth;
            }
        }
        if self.smooth && f64::from(depth) < 1e-4 {
            depth = minimum;
            if self.rib {
                self.width = depth;
            }
        }
        if f64::from(depth) + 1e-4 < f64::from(minimum) {
            if self.rib {
                self.rib_length = self.rib_length.max(minimum * 2.0_f32.sqrt());
            } else {
                self.spacing = (minimum / depth).max(self.spacing);
            }
        }
        for (index, layer) in self.layers.iter_mut().enumerate() {
            apply_spacing(layer, index, self.spacing, self.perimeter, self.width);
        }
        self.update_depth(depth);
        let diagonal = (self.depth * self.depth + self.width * self.width).sqrt();
        self.rib_length = self.rib_length.max(diagonal);
        self.rib_length += self.extra_rib;
        self.rib_length = diagonal.max(self.rib_length);
        self.rib_width = self.rib_width.min(self.depth.min(self.width) / 2.0);
    }
}

fn apply_spacing(layer: &mut Layer, index: usize, spacing: f32, perimeter: f32, width: f32) {
    if index == 0 {
        layer.spacing = 1.0;
        for change in &mut layer.changes {
            let wipe_depth = (change.depth - change.nozzle_depth) * spacing;
            let lines = wipe_depth / perimeter;
            change.nozzle_depth *= spacing;
            change.depth = wipe_depth + change.nozzle_depth;
            change.length = lines * (width - 2.0 * perimeter);
            change.volume =
                super::super::plan::length_to_volume(change.length, perimeter, layer.height);
        }
    } else {
        layer.spacing = spacing;
        for change in &mut layer.changes {
            change.depth *= spacing;
            change.nozzle_depth *= spacing;
            change.length =
                super::super::plan::volume_to_length(change.volume, perimeter, layer.height);
        }
    }
}

fn minimum_depth(height: f32) -> f32 {
    let points = [(5.0, 5.0), (100.0, 20.0), (250.0, 40.0), (350.0, 60.0)];
    for (index, &(h, depth)) in points.iter().enumerate() {
        if h >= height || index == points.len() - 1 {
            return depth;
        }
        let (next_h, next_depth) = points[index + 1];
        if next_h > height {
            return depth + (height - h) / (next_h - h) * (next_depth - depth);
        }
    }
    unreachable!()
}
