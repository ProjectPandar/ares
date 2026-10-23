//! Non-BBL `WipeTower::ToolChangeResult` and `WipeTower2::merge_tcr`.
use super::writer::Extrusion;

pub(super) enum Kind {
    Priming,
    Finish,
    Change { contact: bool },
}

#[derive(Clone)]
pub(super) struct ToolChangeResult {
    pub(super) type1: Option<super::type1::Metadata>,
    pub(super) priming: bool,
    pub(super) initial_tool: usize,
    pub(super) new_tool: usize,
    pub(super) print_z: f32,
    pub(super) layer_height: f32,
    pub(super) start: [f32; 2],
    pub(super) end: [f32; 2],
    pub(super) gcode: String,
    pub(super) extrusions: Vec<Extrusion>,
    pub(super) wipe_path: Vec<[f32; 2]>,
    pub(super) contact: bool,
    pub(super) force_travel: bool,
}

impl ToolChangeResult {
    pub(super) fn extrusion_length(&self) -> f32 {
        let mut length = 0.0;
        for pair in self.extrusions.windows(2) {
            if pair[1].width > 0.0 {
                let dx = pair[1].position[0] - pair[0].position[0];
                let dy = pair[1].position[1] - pair[0].position[1];
                length += (dx * dx + dy * dy).sqrt();
            }
        }
        length
    }

    pub(super) fn merge(mut self, second: Self) -> Self {
        use std::fmt::Write as _;
        self.contact |= second.contact;
        if self.end != second.start {
            writeln!(
                self.gcode,
                "G1 X{:.3} Y{:.3} F7200",
                second.start[0], second.start[1]
            )
            .unwrap();
        }
        self.gcode.push_str(&second.gcode);
        self.extrusions.extend(second.extrusions);
        self.end = second.end;
        self.wipe_path = second.wipe_path;
        self.new_tool = second.new_tool;
        self
    }
}
