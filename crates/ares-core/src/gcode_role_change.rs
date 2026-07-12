use crate::{PrintPathRole, SliceError, SliceOptions, ToolpathMoveKind};

#[derive(Default)]
pub(crate) struct RoleChangeGCodeState {
    last_print_role: Option<PrintPathRole>,
}

pub(crate) struct RoleChangeGCodeCommand<'a> {
    pub(crate) options: &'a SliceOptions,
    pub(crate) move_kind: ToolpathMoveKind,
    pub(crate) role: PrintPathRole,
    pub(crate) layer_num: usize,
    pub(crate) layer_z: &'a str,
}

impl RoleChangeGCodeState {
    pub(crate) const fn new() -> Self {
        Self {
            last_print_role: None,
        }
    }

    pub(crate) fn before_move(
        &mut self,
        command: RoleChangeGCodeCommand<'_>,
    ) -> Result<String, SliceError> {
        if command.move_kind != ToolpathMoveKind::Print {
            return Ok(String::new());
        }

        let previous_role = self.last_print_role.replace(command.role);
        let Some(previous_role) = previous_role else {
            return Ok(String::new());
        };
        if previous_role == command.role {
            return Ok(String::new());
        }

        let placeholder_command = crate::gcode_placeholders::ChangeExtrusionRoleGCodeCommand {
            layer_num: command.layer_num,
            layer_z: command.layer_z,
            extrusion_role: command.role.as_str(),
            last_extrusion_role: previous_role.as_str(),
        };
        let mut gcode = crate::gcode_placeholders::change_extrusion_role_gcode(
            command.options,
            placeholder_command,
        )?;
        gcode.push_str(
            &crate::gcode_placeholders::filament_change_extrusion_role_gcode(
                command.options,
                placeholder_command,
            )?,
        );
        gcode.push_str(
            &crate::gcode_placeholders::process_change_extrusion_role_gcode(
                command.options,
                placeholder_command,
            )?,
        );
        Ok(gcode)
    }
}
