use crate::{
    Point2, ToolpathMoveKind,
    gcode_lift::{TravelLiftMove, slope_lift_move, spiral_lift_move},
    gcode_writer::GCodeWriter,
    options::ZHopLiftMode,
};

#[derive(Clone, Copy)]
pub(crate) struct LayerChangeRetractCommand {
    pub(crate) use_firmware: bool,
    pub(crate) length: f64,
    pub(crate) feedrate: f64,
}

#[derive(Clone, Copy)]
pub(crate) struct LayerChangeUnretractCommand {
    pub(crate) use_firmware: bool,
    pub(crate) length: f64,
    pub(crate) unretract_length: f64,
    pub(crate) feedrate: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct PendingLayerChangeLift {
    z_restore: f64,
    z_hop: f64,
    z_hop_lift: PendingLayerChangeLiftMode,
    resolution: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum PendingLayerChangeLiftMode {
    Slope { radians: f64 },
    Spiral { radians: f64 },
}

#[derive(Debug)]
pub(crate) struct LayerChangeLiftState {
    pending_z_restore: Option<f64>,
    pending_lift: Option<PendingLayerChangeLift>,
}

pub(crate) struct LayerChangeLiftCommand<'a> {
    pub(crate) writer: &'a mut GCodeWriter,
    pub(crate) z_hop: f64,
    pub(crate) z_hop_lift: ZHopLiftMode,
    pub(crate) resolution: f64,
    pub(crate) feedrate: f64,
    pub(crate) comment: Option<&'a str>,
}

pub(crate) struct LayerChangeRestoreCommand<'a> {
    pub(crate) writer: &'a mut GCodeWriter,
    pub(crate) feedrate: f64,
    pub(crate) lift_comment: Option<&'a str>,
    pub(crate) restore_comment: Option<&'a str>,
}

pub(crate) struct LayerChangeResumeCommand<'a> {
    pub(crate) writer: &'a mut GCodeWriter,
    pub(crate) lift_state: &'a mut LayerChangeLiftState,
    pub(crate) pending_unretract: &'a mut bool,
    pub(crate) use_firmware: bool,
    pub(crate) length: f64,
    pub(crate) unretract_length: f64,
    pub(crate) unretract_feedrate: f64,
    pub(crate) kind: ToolpathMoveKind,
    pub(crate) z_feedrate: f64,
    pub(crate) lift_comment: Option<&'a str>,
    pub(crate) restore_comment: Option<&'a str>,
    pub(crate) unretract_comment: Option<&'a str>,
}

impl LayerChangeLiftState {
    pub(crate) const fn new() -> Self {
        Self {
            pending_z_restore: None,
            pending_lift: None,
        }
    }

    pub(crate) fn schedule_lift(&mut self, command: LayerChangeLiftCommand<'_>) -> String {
        self.pending_z_restore = None;
        self.pending_lift = None;
        if command.z_hop <= 0.0 {
            return String::new();
        }

        let z_restore = command.writer.current_position().2;
        let raised_z = z_restore + command.z_hop;
        self.pending_z_restore = Some(z_restore);
        match command.z_hop_lift {
            ZHopLiftMode::Normal => {
                command
                    .writer
                    .travel_to_z_with_comment(raised_z, command.feedrate, command.comment)
            }
            ZHopLiftMode::Slope { radians } => {
                self.pending_lift = Some(PendingLayerChangeLift {
                    z_restore,
                    z_hop: command.z_hop,
                    z_hop_lift: PendingLayerChangeLiftMode::Slope { radians },
                    resolution: command.resolution,
                });
                String::new()
            }
            ZHopLiftMode::Auto { radians } | ZHopLiftMode::Spiral { radians } => {
                self.pending_lift = Some(PendingLayerChangeLift {
                    z_restore,
                    z_hop: command.z_hop,
                    z_hop_lift: PendingLayerChangeLiftMode::Spiral { radians },
                    resolution: command.resolution,
                });
                String::new()
            }
        }
    }

    pub(crate) fn consume_travel_lift(
        &mut self,
        writer: &GCodeWriter,
        kind: ToolpathMoveKind,
        target: Point2,
    ) -> Option<TravelLiftMove> {
        if kind != ToolpathMoveKind::Travel {
            return None;
        }
        let pending = self.pending_lift.take()?;
        let raised_z = pending.z_restore + pending.z_hop;
        Some(match pending.z_hop_lift {
            PendingLayerChangeLiftMode::Slope { radians } => {
                slope_lift_move(writer, target, pending.z_hop, radians, raised_z)
            }
            PendingLayerChangeLiftMode::Spiral { radians } => {
                spiral_lift_move(writer, target, radians, pending.resolution, raised_z)
            }
        })
    }

    pub(crate) fn restore_before_unretract(
        &mut self,
        command: LayerChangeRestoreCommand<'_>,
    ) -> String {
        let mut gcode = String::new();
        if let Some(pending) = self.pending_lift.take() {
            gcode.push_str(&command.writer.travel_to_z_with_comment(
                pending.z_restore + pending.z_hop,
                command.feedrate,
                command.lift_comment,
            ));
        }
        gcode.push_str(&layer_change_z_restore_gcode(
            command.writer,
            &mut self.pending_z_restore,
            command.feedrate,
            command.restore_comment,
        ));
        gcode
    }
}

pub(crate) fn layer_change_z_restore_gcode(
    writer: &mut GCodeWriter,
    pending_z_restore: &mut Option<f64>,
    feedrate: f64,
    comment: Option<&str>,
) -> String {
    let Some(z_restore) = pending_z_restore.take() else {
        return String::new();
    };
    writer.travel_to_z_with_comment(z_restore, feedrate, comment)
}

pub(crate) fn layer_change_retract_gcode(
    writer: &mut GCodeWriter,
    command: LayerChangeRetractCommand,
    comment: Option<&str>,
) -> String {
    if command.use_firmware {
        writer.firmware_retract()
    } else {
        writer.retract_with_comment(command.length, command.feedrate, comment)
    }
}

pub(crate) fn layer_change_unretract_gcode(
    writer: &mut GCodeWriter,
    command: LayerChangeUnretractCommand,
    comment: Option<&str>,
) -> (String, f64) {
    if command.use_firmware {
        (writer.firmware_unretract(), 0.0)
    } else {
        (
            writer.unretract_with_comment(command.unretract_length, command.feedrate, comment),
            command.unretract_length - command.length,
        )
    }
}

pub(crate) fn layer_change_resume_before_print(
    command: LayerChangeResumeCommand<'_>,
) -> (String, f64) {
    if !*command.pending_unretract || command.kind != ToolpathMoveKind::Print {
        return (String::new(), 0.0);
    }

    let writer = command.writer;
    let mut gcode = command
        .lift_state
        .restore_before_unretract(LayerChangeRestoreCommand {
            writer: &mut *writer,
            feedrate: command.z_feedrate,
            lift_comment: command.lift_comment,
            restore_comment: command.restore_comment,
        });
    let (unretract_gcode, e_offset_delta) = layer_change_unretract_gcode(
        writer,
        LayerChangeUnretractCommand {
            use_firmware: command.use_firmware,
            length: command.length,
            unretract_length: command.unretract_length,
            feedrate: command.unretract_feedrate,
        },
        command.unretract_comment,
    );
    gcode.push_str(&unretract_gcode);
    *command.pending_unretract = false;
    (gcode, e_offset_delta)
}
