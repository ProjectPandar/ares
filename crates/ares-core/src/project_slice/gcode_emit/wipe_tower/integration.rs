//! `GCode.hpp::WipeTowerIntegration` and `GCode.cpp:1497–1534` Type2 cursor.
use super::{Data, filament::at};
use crate::ProjectSettings;
use std::rc::Rc;

#[derive(Clone, Copy, PartialEq)]
pub(super) enum Priming {
    Disabled,
    Clear,
    Overlapping,
}

pub(in crate::project_slice::gcode_emit) struct Integration {
    pub(super) data: Rc<Data>,
    pub(super) position: [f32; 2],
    pub(super) angle: f32,
    pub(super) offsets: Vec<[f32; 2]>,
    pub(super) semm: bool,
    pub(super) priming: Priming,
    center_no_tower: [f64; 2],
    no_sparse: bool,
    layer: Option<usize>,
    change: usize,
    last_z: f64,
    first_print: bool,
    extra_wall: bool,
}

impl Integration {
    pub(in crate::project_slice::gcode_emit) fn used_filament(&self, tool: usize) -> f64 {
        f64::from(self.data.used_filament[tool])
    }

    pub(in crate::project_slice::gcode_emit) fn tool_changes(&self) -> usize {
        self.data.tool_changes
    }
    pub(in crate::project_slice::gcode_emit) fn priming_enabled(&self) -> bool {
        self.priming != Priming::Disabled
    }

    pub(in crate::project_slice::gcode_emit) fn insert_placeholders(
        &self,
        config: &mut super::super::value::Config,
    ) {
        use super::super::value::Value;
        config.insert("has_wipe_tower", Value::option_bool(true));
        config.insert(
            "total_toolchanges",
            Value::Integer(self.tool_changes() as i64),
        );
        config.insert(
            "has_single_extruder_multi_material_priming",
            Value::option_bool(self.priming_enabled()),
        );
        config.insert(
            "first_layer_center_no_wipe_tower",
            Value::List(self.center_no_tower.map(Value::number).to_vec()),
        );
    }
    pub(in crate::project_slice::gcode_emit) fn new(
        data: Data,
        settings: &ProjectSettings,
        center_no_tower: [f64; 2],
        priming_overlap: bool,
        position: [f64; 2],
    ) -> Self {
        let project = &settings.project.print;
        Self {
            position: position.map(|value| value as f32),
            angle: settings.process.print.wipe_tower_rotation_angle.0 as f32 / 180.0
                * std::f32::consts::PI,
            offsets: settings
                .project
                .gcode
                .filament_map
                .0
                .iter()
                .map(|id| {
                    let offset = at(&project.extruder_offset.0, (id.0 - 1) as usize);
                    [offset.x as f32, offset.y as f32]
                })
                .collect(),
            semm: settings.printer.gcode.single_extruder_multi_material.0,
            priming: if data.type1
                || !settings
                    .process
                    .gcode
                    .single_extruder_multi_material_priming
                    .0
            {
                Priming::Disabled
            } else if priming_overlap {
                Priming::Overlapping
            } else {
                Priming::Clear
            },
            center_no_tower,
            no_sparse: settings.process.gcode.wipe_tower_no_sparse_layers.0,
            layer: None,
            change: 0,
            first_print: true,
            extra_wall: settings.process.print.timelapse_type
                == crate::ProcessTimelapseType::Smooth
                || settings.process.gcode.enable_wrapping_detection.0,
            last_z: settings.printer.remaining.z_offset.0,
            data: Rc::new(data),
        }
    }

    pub(in crate::project_slice::gcode_emit) fn next_layer(&mut self) {
        self.layer = Some(self.layer.map_or(0, |layer| layer + 1));
        self.change = 0;
        self.first_print = true;
    }

    pub(super) fn first_wall(&mut self) -> Option<(usize, Option<f64>)> {
        let layer = self.layer.expect("tower dispatch follows next_layer");
        let changes = self.data.changes.get(layer)?;
        if !self.data.type1 || !self.extra_wall || !self.first_print {
            return None;
        }
        let ignore = changes.len() == 1 && changes[0].initial_tool == changes[0].new_tool;
        let z = self.no_sparse.then(|| {
            self.last_z
                + if ignore {
                    0.0
                } else {
                    f64::from(changes[0].layer_height)
                }
        });
        self.change += 1;
        self.first_print = false;
        Some((layer, z))
    }

    pub(super) fn next(
        &mut self,
        needs_change: bool,
        finish_layer: bool,
    ) -> Option<(usize, usize, Option<f64>)> {
        let layer = self.layer.expect("tower dispatch follows next_layer");
        let changes = self.data.changes.get(layer)?;
        if !needs_change && !finish_layer {
            return None;
        }
        if self.no_sparse
            && (self.data.type1 || layer != 0)
            && changes.len() == 1
            && changes[0].initial_tool == changes[0].new_tool
        {
            return None;
        }
        let z = self.no_sparse.then(|| {
            self.last_z
                + if self.change == 0 {
                    f64::from(changes[0].layer_height)
                } else {
                    0.0
                }
        });
        let change = self.change;
        self.change += 1;
        self.last_z = z.unwrap_or(-1.0);
        Some((layer, change, z))
    }
}
