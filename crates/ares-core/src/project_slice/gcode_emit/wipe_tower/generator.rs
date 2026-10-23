//! Mutable `WipeTower2` generation state; planning and emission share this state.
use super::{
    filament::Filament, geometry::Shape, plan::Plan, result::ToolChangeResult, writer::Writer,
};
use crate::{
    GCodeFlavor, ProjectSettings,
    geometry::{BoundingBox, CoordinateScale},
};

#[cfg(test)]
mod tests;

pub(super) struct Generator<'a> {
    pub(super) settings: &'a ProjectSettings,
    pub(super) scale: CoordinateScale,
    pub(super) flavor: GCodeFlavor,
    pub(super) filaments: Vec<Filament>,
    pub(super) plan: Plan,
    pub(super) wipe_volumes: Vec<Vec<f32>>,
    pub(super) current_tool: usize,
    pub(super) layer_index: usize,
    pub(super) z: f32,
    pub(super) layer_height: f32,
    pub(super) extrusion_flow: f32,
    pub(super) current_height: f32,
    pub(super) depth_traversed: f32,
    pub(super) layer_finished: bool,
    pub(super) y_shift: f32,
    pub(super) rotation: f32,
    pub(super) previous_interface: bool,
    pub(super) current_interface: bool,
    pub(super) layer_changes: usize,
    pub(super) tool_changes: usize,
    pub(super) left_to_right: bool,
    pub(super) old_temperature: i32,
    pub(super) mk4_mmu3: bool,
    pub(super) used_filament: Vec<f32>,
    pub(super) used_until_layer: Vec<(f32, Vec<f32>)>,
    pub(super) rib_width: f32,
    pub(super) rib_length: f32,
    pub(super) brim_width: f32,
    pub(super) first_layer_bounds: Option<BoundingBox>,
}

impl<'a> Generator<'a> {
    pub(super) fn new(
        settings: &'a ProjectSettings,
        filaments: Vec<Filament>,
        (scale, flavor): (CoordinateScale, GCodeFlavor),
        initial_tool: usize,
        wipe_volumes: Vec<Vec<f32>>,
    ) -> Self {
        let notes = settings
            .printer
            .remaining
            .printer_notes
            .0
            .to_ascii_uppercase();
        let plan = Plan::new(settings, &filaments);
        let used_filament = vec![0.0; filaments.len()];
        Self {
            settings,
            scale,
            flavor,
            filaments,
            plan,
            wipe_volumes,
            current_tool: initial_tool,
            layer_index: 0,
            z: 0.0,
            layer_height: 0.0,
            extrusion_flow: 0.038,
            current_height: 0.0,
            depth_traversed: 0.0,
            layer_finished: false,
            y_shift: 0.0,
            rotation: 0.0,
            previous_interface: false,
            current_interface: false,
            layer_changes: 0,
            tool_changes: 0,
            left_to_right: true,
            old_temperature: -1,
            mk4_mmu3: notes.contains("PRINTER_MODEL_MK4") && notes.contains("MMU"),
            used_filament,
            used_until_layer: Vec::new(),
            rib_width: settings.process.print.wipe_tower_rib_width.0 as f32,
            rib_length: 0.0,
            brim_width: 0.0,
            first_layer_bounds: None,
        }
    }

    pub(super) fn is_first_layer(&self) -> bool {
        self.plan.first_layer == Some(self.layer_index)
    }

    pub(super) fn flow_for_height(&self, height: f32) -> f32 {
        height * (self.plan.perimeter_width - height * (1.0 - std::f32::consts::PI / 4.0))
            / self.filaments[0].area
    }

    pub(super) fn set_layer(&mut self, z: f32, height: f32) {
        self.z = z;
        self.layer_height = height;
        self.depth_traversed = 0.0;
        self.layer_finished = false;
        self.previous_interface = self.current_interface;
        while self.layer_index + 1 < self.plan.layers.len()
            && self.plan.layers[self.layer_index].z < z - 1e-3
        {
            self.layer_index += 1;
        }
        self.current_interface = self
            .plan
            .layers
            .get(self.layer_index)
            .is_some_and(|layer| layer.changes_depth() > 1e-3);
        if self.is_first_layer() {
            self.layer_changes = 0;
            self.tool_changes = 0;
        } else {
            self.layer_changes += 1;
        }
        self.extrusion_flow = self.flow_for_height(height);
    }

    pub(super) fn writer(&self) -> Writer {
        let mut writer = Writer::new(
            self.layer_height,
            self.plan.perimeter_width,
            self.flavor,
            self.filaments[0].area,
        );
        writer.set_flow(self.extrusion_flow);
        writer.set_tool(
            self.current_tool,
            self.filaments[self.current_tool].max_e_speed,
        );
        writer
    }

    pub(super) fn result(
        &self,
        writer: Writer,
        old_tool: usize,
        kind: super::result::Kind,
    ) -> ToolChangeResult {
        let (priming, contact) = match kind {
            super::result::Kind::Priming => (true, false),
            super::result::Kind::Finish => (false, false),
            super::result::Kind::Change { contact } => (false, contact),
        };
        let end = if priming {
            writer.position
        } else {
            writer.rotated(writer.position)
        };
        ToolChangeResult {
            type1: None,
            priming,
            initial_tool: old_tool,
            new_tool: self.current_tool,
            print_z: self.z,
            layer_height: self.layer_height,
            start: writer.start,
            end,
            gcode: writer.gcode,
            extrusions: writer.extrusions,
            wipe_path: writer.wipe_path,
            contact,
            force_travel: false,
        }
    }

    pub(super) fn shape(&self) -> Shape {
        Shape {
            width: self.plan.width,
            depth: self.plan.depth,
            height: self.plan.height,
            z: self.z,
            y_shift: self.y_shift,
            rib_width: self.rib_width,
            rib_length: self.rib_length,
            fillet: self.settings.process.print.wipe_tower_fillet_wall.0,
        }
    }

    pub(super) fn first_nonsoluble_change(&self) -> Option<usize> {
        let changes = &self.plan.layers[self.layer_index].changes;
        if self.settings.process.print.wipe_tower_filament.0 > 0 {
            changes
                .iter()
                .position(|change| !self.filaments[change.new_tool].soluble)
        } else {
            (!changes.is_empty()).then_some(0)
        }
    }

    pub(super) fn first_layer_speed(&self) -> f32 {
        let speed = self.settings.process.print.initial_layer_speed.0 as f32;
        if speed == 0.0 { 30.0 } else { speed }
    }

    pub(super) fn perimeter_speed(&self) -> f32 {
        let speed = self.settings.process.region.inner_wall_speed.0 as f32;
        if speed == 0.0 { 80.0 } else { speed }
    }

    pub(super) fn infill_speed(&self) -> f32 {
        let speed = self.settings.process.region.sparse_infill_speed.0 as f32;
        if speed == 0.0 { 80.0 } else { speed }
    }
}
