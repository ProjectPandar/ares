use crate::{
    Point2, PrintPathRole, SpeedMove, ToolpathMoveKind,
    gcode_writer::GCodeWriter,
    options::{FanSpeedupControl, LayerRoleFanControl},
};

const FAN_KICKSTART_THRESHOLD_PERCENT: u8 = 10;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct RoleFanGCodeState {
    current_speed: Option<u8>,
    physical_speed: Option<u8>,
    role_override_active: bool,
    fan_kickstart_s: f64,
    fan_speedup: FanSpeedupControl,
    pending: Option<PendingFanKickstart>,
    last_point: Option<Point2>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RoleFanCommand {
    pub(crate) gcode: String,
    pub(crate) speedup_eligible: bool,
}

impl RoleFanCommand {
    fn new(gcode: String, speedup_eligible: bool) -> Self {
        Self {
            gcode,
            speedup_eligible,
        }
    }

    pub(crate) fn empty() -> Self {
        Self::new(String::new(), false)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct PendingFanKickstart {
    target_speed: u8,
    remaining_s: f64,
}

#[derive(Clone, Copy)]
pub(crate) struct RoleFanMoveCommand<'a> {
    pub(crate) writer: &'a GCodeWriter,
    pub(crate) role_fan_control: LayerRoleFanControl,
    pub(crate) baseline_speed: Option<u8>,
    pub(crate) move_kind: ToolpathMoveKind,
    pub(crate) role: PrintPathRole,
}

impl RoleFanGCodeState {
    pub(crate) const fn new(
        current_speed: Option<u8>,
        fan_kickstart_s: f64,
        fan_speedup: FanSpeedupControl,
    ) -> Self {
        Self {
            current_speed,
            physical_speed: current_speed,
            role_override_active: false,
            fan_kickstart_s,
            fan_speedup,
            pending: None,
            last_point: None,
        }
    }

    pub(crate) fn layer_baseline_command(&mut self, writer: &GCodeWriter, speed: u8) -> String {
        let command = self.set_speed(writer, speed);
        self.role_override_active = false;
        command
    }

    pub(crate) fn before_move(&mut self, command: RoleFanMoveCommand<'_>) -> RoleFanCommand {
        if command.move_kind != ToolpathMoveKind::Print {
            return RoleFanCommand::empty();
        }
        if let Some(override_speed) = command.role_fan_control.speed_for_role(command.role) {
            let speedup_eligible = self.speedup_eligible(command.role, override_speed);
            self.role_override_active = true;
            let gcode = self.set_speed(command.writer, override_speed);
            let emitted = !gcode.is_empty();
            return RoleFanCommand::new(gcode, speedup_eligible && emitted);
        }
        if !self.role_override_active {
            return RoleFanCommand::empty();
        }
        self.role_override_active = false;
        let gcode = match command.baseline_speed {
            Some(speed) => self.set_speed(command.writer, speed),
            None => self.set_speed(command.writer, 0),
        };
        RoleFanCommand::new(gcode, false)
    }

    pub(crate) fn can_speedup_before_move(&self, command: &RoleFanMoveCommand<'_>) -> bool {
        if command.move_kind != ToolpathMoveKind::Print {
            return false;
        }
        let Some(override_speed) = command.role_fan_control.speed_for_role(command.role) else {
            return false;
        };
        if !self.speedup_eligible(command.role, override_speed) {
            return false;
        }
        let mut state = *self;
        !state.set_speed(command.writer, override_speed).is_empty()
    }

    pub(crate) fn after_move(&mut self, writer: &GCodeWriter, speed_move: &SpeedMove) -> String {
        let command = match self.move_time_s(speed_move) {
            Some(move_time_s) => self.advance_pending(writer, move_time_s),
            None => String::new(),
        };
        self.last_point = Some(speed_move.point());
        command
    }

    pub(crate) fn finish(&mut self, writer: &GCodeWriter) -> String {
        match self.pending.take() {
            Some(pending) => self.emit_physical_speed(writer, pending.target_speed),
            None => String::new(),
        }
    }

    fn set_speed(&mut self, writer: &GCodeWriter, speed: u8) -> String {
        let previous_speed = self.current_speed.unwrap_or(0);
        self.current_speed = Some(speed);
        if let Some(mut pending) = self.pending {
            if speed > pending.target_speed {
                pending.remaining_s +=
                    self.fan_kickstart_s * f64::from(speed - pending.target_speed) / 100.0;
                pending.target_speed = speed;
                self.pending = Some(pending);
                return self.emit_physical_speed(writer, 100);
            }
            self.pending = None;
            return self.emit_physical_speed(writer, speed);
        }
        if speed == 0 {
            return self.emit_physical_speed(writer, 0);
        }
        if self.fan_kickstart_s <= 0.0
            || speed <= previous_speed.saturating_add(FAN_KICKSTART_THRESHOLD_PERCENT)
        {
            return self.emit_physical_speed(writer, speed);
        }
        self.pending = Some(PendingFanKickstart {
            target_speed: speed,
            remaining_s: self.fan_kickstart_s * f64::from(speed - previous_speed) / 100.0,
        });
        self.emit_physical_speed(writer, 100)
    }

    fn speedup_eligible(&self, role: PrintPathRole, speed: u8) -> bool {
        speed > self.current_speed.unwrap_or(0) && self.fan_speedup.applies_to_role(role)
    }

    fn advance_pending(&mut self, writer: &GCodeWriter, move_time_s: f64) -> String {
        let Some(mut pending) = self.pending else {
            return String::new();
        };
        pending.remaining_s -= move_time_s;
        if pending.remaining_s > 0.0 {
            self.pending = Some(pending);
            return String::new();
        }
        self.pending = None;
        self.emit_physical_speed(writer, pending.target_speed)
    }

    fn move_time_s(&self, speed_move: &SpeedMove) -> Option<f64> {
        let start = self.last_point?;
        let speed = speed_move.speed_mm_s();
        if speed <= 0.0 {
            return None;
        }
        let length = distance(start, speed_move.point());
        (length > 0.0).then_some(length / speed)
    }

    fn emit_physical_speed(&mut self, writer: &GCodeWriter, speed: u8) -> String {
        if self.physical_speed == Some(speed) {
            String::new()
        } else {
            self.physical_speed = Some(speed);
            writer.set_fan(speed)
        }
    }
}

fn distance(start: Point2, end: Point2) -> f64 {
    ((end.x() - start.x()).powi(2) + (end.y() - start.y()).powi(2)).sqrt()
}
