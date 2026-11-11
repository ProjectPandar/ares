//! `WipeTower.cpp:1240–1292,2660–2707`: Type1 result construction/merge.
use super::super::result::ToolChangeResult;
use super::writer::Writer;
use std::fmt::Write;

#[derive(Clone, Default)]
pub(super) struct NozzleChange {
    pub(super) gcode: String,
    pub(super) start: [f32; 2],
    pub(super) end: [f32; 2],
    pub(super) origin_start: [f32; 2],
    pub(super) wipe_path: Vec<[f32; 2]>,
}

pub(super) struct Result {
    pub(super) common: ToolChangeResult,
    pub(super) finish_first: bool,
    pub(super) tool_change: bool,
    pub(in crate::project_slice::gcode_emit::wipe_tower) change_start: [f32; 2],
    pub(super) nozzle_change: NozzleChange,
    pub(super) purge: f32,
}

#[derive(Clone)]
pub(in crate::project_slice::gcode_emit::wipe_tower) struct Metadata {
    pub(super) finish_first: bool,
    pub(super) tool_change: bool,
    pub(in crate::project_slice::gcode_emit::wipe_tower) change_start: [f32; 2],
    pub(super) nozzle_change: NozzleChange,
    pub(super) purge: f32,
}

impl Result {
    pub(super) fn into_common(mut self) -> ToolChangeResult {
        self.common.type1 = Some(Metadata {
            finish_first: self.finish_first,
            tool_change: self.tool_change,
            change_start: self.change_start,
            nozzle_change: self.nozzle_change,
            purge: self.purge,
        });
        self.common
    }
    pub(super) fn from_writer(
        writer: Writer,
        z: f32,
        height: f32,
        old: usize,
        finish: bool,
    ) -> Self {
        let end = writer.rotated(writer.position);
        Self {
            common: ToolChangeResult {
                type1: None,
                priming: false,
                initial_tool: old,
                new_tool: writer.tool,
                print_z: z,
                layer_height: height,
                start: writer.start,
                end,
                gcode: writer.gcode,
                extrusions: writer.extrusions,
                wipe_path: writer.wipe_path,
                contact: false,
                force_travel: false,
            },
            finish_first: finish,
            tool_change: false,
            change_start: [0.0; 2],
            nozzle_change: NozzleChange::default(),
            purge: 0.0,
        }
    }

    pub(super) fn merge(mut self, second: Self, max_speed: f32) -> Self {
        self.common.contact |= second.common.contact;
        let dx = self.common.end[0] - second.common.start[0];
        let dy = self.common.end[1] - second.common.start[1];
        if (dx * dx + dy * dy).sqrt() > 1e-4_f32 {
            let change_at_start = second.tool_change
                && (second.common.start[0] - second.change_start[0]).abs() < 1e-4
                && (second.common.start[1] - second.change_start[1]).abs() < 1e-4;
            if !change_at_start {
                writeln!(
                    self.common.gcode,
                    "G1 X{:.3} Y{:.3} F{max_speed:.6}",
                    second.common.start[0], second.common.start[1]
                )
                .unwrap();
            }
        }
        self.common.gcode.push_str(&second.common.gcode);
        self.common.extrusions.extend(second.common.extrusions);
        self.common.end = second.common.end;
        self.common.wipe_path = second.common.wipe_path;
        self.common.new_tool = second.common.new_tool;
        if self.nozzle_change.gcode.is_empty() && !second.nozzle_change.gcode.is_empty() {
            self.nozzle_change = second.nozzle_change;
        }
        if !self.tool_change && second.tool_change {
            self.tool_change = true;
            self.change_start = second.change_start;
        }
        self.purge += second.purge;
        self
    }
}
