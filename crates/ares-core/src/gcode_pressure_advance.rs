// Source: OrcaSlicer/src/libslic3r/GCode.cpp
// Source: OrcaSlicer/src/libslic3r/GCodeWriter.cpp

use crate::{PrintPathRole, SliceError, SliceOptions, ToolpathMoveKind, gcode_writer::GCodeWriter};

pub(crate) fn startup_command(
    writer: &GCodeWriter,
    options: &SliceOptions,
) -> Result<String, SliceError> {
    Ok(options
        .pressure_advance_control()?
        .value()
        .map(|value| writer.set_pressure_advance(value))
        .unwrap_or_default())
}

pub(crate) struct PressureAdvanceMoveState {
    base_value: Option<f64>,
    bridge_value: Option<f64>,
    bridge_active: bool,
}

impl PressureAdvanceMoveState {
    pub(crate) fn from_options(options: &SliceOptions) -> Result<Self, SliceError> {
        let control = options.pressure_advance_control()?;
        Ok(Self {
            base_value: control.value(),
            bridge_value: control.bridge_value(),
            bridge_active: false,
        })
    }

    pub(crate) fn before_move(
        &mut self,
        writer: &GCodeWriter,
        kind: ToolpathMoveKind,
        role: PrintPathRole,
    ) -> String {
        if kind != ToolpathMoveKind::Print {
            return String::new();
        }
        let wants_bridge = self.bridge_value.is_some() && bridge_pressure_advance_role(role);
        match (self.bridge_active, wants_bridge) {
            (false, true) => {
                self.bridge_active = true;
                self.bridge_value
                    .map(|value| writer.set_pressure_advance(value))
                    .unwrap_or_default()
            }
            (true, false) => {
                self.bridge_active = false;
                self.base_value
                    .map(|value| writer.set_pressure_advance(value))
                    .unwrap_or_default()
            }
            _ => String::new(),
        }
    }
}

fn bridge_pressure_advance_role(role: PrintPathRole) -> bool {
    matches!(
        role,
        PrintPathRole::Bridge | PrintPathRole::InternalBridge | PrintPathRole::OverhangPerimeter
    )
}
