//! `WipeTower.cpp::generate_new` and `WipeTower.hpp::set_layer` state.
use super::{
    filament::Filament,
    plan::Plan,
    result::{NozzleChange, Result},
    writer::Writer,
};
use crate::{
    ProjectSettings,
    geometry::{CoordinateScale, Polygon},
};

pub(super) struct Generator<'a> {
    pub(super) settings: &'a ProjectSettings,
    pub(super) scale: CoordinateScale,
    pub(super) plan: Plan,
    pub(super) filaments: Vec<Filament>,
    pub(super) current: usize,
    pub(super) index: usize,
    pub(super) z: f32,
    pub(super) height: f32,
    pub(super) flow: f32,
    pub(super) y_shift: f32,
    pub(super) left_to_right: bool,
    pub(super) changes: usize,
    pub(super) used: Vec<f32>,
    pub(super) used_tools: Vec<usize>,
    pub(super) nozzle_change: NozzleChange,
    pub(super) outer_walls: Vec<Vec<Polygon>>,
    pub(super) rib_offset: [f32; 2],
    pub(super) brim: f32,
    pub(super) finished: bool,
    pub(super) current_block: Option<usize>,
    pub(super) wall_gaps: Vec<[f32; 2]>,
}

impl<'a> Generator<'a> {
    pub(super) fn new(
        settings: &'a ProjectSettings,
        scale: CoordinateScale,
        planned: super::planning::Planned,
    ) -> Self {
        let count = settings.filament.gcode.filament_colour.0.len();
        Self {
            settings,
            scale,
            plan: planned.plan,
            filaments: (0..count).map(|i| Filament::new(settings, i)).collect(),
            current: planned.first_tool,
            index: 0,
            z: 0.0,
            height: 0.0,
            flow: 0.0,
            y_shift: 0.0,
            left_to_right: true,
            changes: 0,
            used: vec![0.0; count],
            used_tools: planned.used,
            nozzle_change: NozzleChange::default(),
            outer_walls: Vec::new(),
            rib_offset: [0.0; 2],
            brim: 0.0,
            finished: false,
            current_block: None,
            wall_gaps: Vec::new(),
        }
    }

    pub(super) fn first_layer(&self) -> bool {
        self.plan.first_layer == Some(self.index)
    }

    pub(super) fn flow_at(&self, height: f32) -> f32 {
        height * (self.plan.perimeter - height * (1.0 - std::f32::consts::PI / 4.0))
            / self.filaments[0].area
    }

    pub(super) fn first_speed(&self) -> f32 {
        let speed = self.settings.process.print.initial_layer_speed.0 as f32;
        if speed == 0.0 { 30.0 } else { speed }
    }

    pub(super) fn finish_speed(&self, tool: usize) -> f32 {
        if self.first_layer() {
            (self.first_speed() * 60.0).min(5400.0)
        } else {
            (60.0 * self.filaments[tool].max_e_speed / self.flow).min(5400.0)
        }
    }

    pub(super) fn writer(&self, tool: usize) -> Writer {
        let mut writer = Writer::new(
            self.height,
            self.plan.perimeter,
            self.filaments[0].area,
            tool,
        );
        writer.flow = self.flow;
        writer.max_e_speed = self.filaments[tool].max_e_speed;
        writer.set_y_shift(self.y_shift);
        writer
    }

    pub(super) fn finish_result(&mut self, writer: Writer, tool: usize) -> Result {
        if !self.settings.process.gcode.wipe_tower_no_sparse_layers.0
            || self.plan.layers[self.index]
                .changes
                .iter()
                .map(|c| c.depth)
                .sum::<f32>()
                > 1e-3
        {
            self.used[tool] += writer.used;
        }
        Result::from_writer(writer, self.z, self.height, tool, true)
    }
}
