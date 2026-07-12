use crate::{
    Point2, PrintPathRole, ToolpathMoveKind,
    gcode_layer_change_retraction::{
        LayerChangeRetractCommand, LayerChangeUnretractCommand, layer_change_retract_gcode,
        layer_change_unretract_gcode, layer_change_z_restore_gcode,
    },
    gcode_lift::{TravelLiftMove, distance, slope_lift_move, spiral_lift_move},
    gcode_writer::GCodeWriter,
    options::{RetractLiftEnforce, ZHopLiftMode},
};

#[derive(Clone, Copy, Debug, PartialEq)]
struct PreviousPrintSegment {
    start: Point2,
    end: Point2,
    feedrate: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct RetractionSplitInput {
    length: f64,
    retract_before_wipe: f64,
    retract_feedrate: f64,
    wipe_feedrate: f64,
    wipe_distance: f64,
    segment: PreviousPrintSegment,
}

pub(crate) struct TravelRetractionCommand<'a> {
    pub(crate) writer: &'a mut GCodeWriter,
    pub(crate) use_firmware: bool,
    pub(crate) length: f64,
    pub(crate) retract_feedrate: f64,
    pub(crate) minimum_travel: f64,
    pub(crate) z_hop: f64,
    pub(crate) lift_enforce: RetractLiftEnforce,
    pub(crate) current_layer_is_first: bool,
    pub(crate) previous_non_gap_fill_role: Option<PrintPathRole>,
    pub(crate) kind: ToolpathMoveKind,
    pub(crate) role: PrintPathRole,
    pub(crate) target: Point2,
    pub(crate) pending_layer_change_unretract: bool,
    pub(crate) travel_retraction_enabled: bool,
    pub(crate) reduce_infill_retraction: bool,
    pub(crate) sparse_infill_density_positive: bool,
    pub(crate) wipe: bool,
    pub(crate) wipe_distance: f64,
    pub(crate) retract_before_wipe: f64,
    pub(crate) role_based_wipe_speed: bool,
    pub(crate) wipe_feedrate: f64,
    pub(crate) z_feedrate: f64,
    pub(crate) retract_comment: Option<&'a str>,
    pub(crate) z_lift_comment: Option<&'a str>,
}

pub(crate) struct TravelUnretractCommand<'a> {
    pub(crate) writer: &'a mut GCodeWriter,
    pub(crate) use_firmware: bool,
    pub(crate) length: f64,
    pub(crate) unretract_length: f64,
    pub(crate) unretract_feedrate: f64,
    pub(crate) kind: ToolpathMoveKind,
    pub(crate) z_feedrate: f64,
    pub(crate) z_restore_comment: Option<&'a str>,
    pub(crate) unretract_comment: Option<&'a str>,
}

#[derive(Debug)]
pub(crate) struct TravelRetractionState {
    has_printed_move: bool,
    pending_unretract: bool,
    pending_z_restore: Option<f64>,
    pending_travel_lift: Option<TravelLiftMove>,
    previous_print_segment: Option<PreviousPrintSegment>,
    z_hop_lift: ZHopLiftMode,
    resolution: f64,
}

impl TravelRetractionState {
    pub(crate) const fn new(z_hop_lift: ZHopLiftMode, resolution: f64) -> Self {
        Self {
            has_printed_move: false,
            pending_unretract: false,
            pending_z_restore: None,
            pending_travel_lift: None,
            previous_print_segment: None,
            z_hop_lift,
            resolution,
        }
    }

    pub(crate) fn pending_unretract(&self) -> bool {
        self.pending_unretract
    }

    pub(crate) fn consume_travel_lift(&mut self) -> Option<TravelLiftMove> {
        self.pending_travel_lift.take()
    }

    pub(crate) fn retract_before_travel(&mut self, command: TravelRetractionCommand<'_>) -> String {
        self.pending_travel_lift = None;
        if !self.should_retract(&command) {
            return String::new();
        }

        let TravelRetractionCommand {
            writer,
            use_firmware,
            length,
            retract_feedrate,
            z_hop,
            lift_enforce,
            current_layer_is_first,
            previous_non_gap_fill_role,
            wipe,
            wipe_distance,
            retract_before_wipe,
            role_based_wipe_speed,
            wipe_feedrate,
            z_feedrate,
            retract_comment,
            z_lift_comment,
            target,
            ..
        } = command;
        let wipe_segment = (wipe && wipe_distance > 0.0 && !use_firmware)
            .then_some(())
            .and(self.previous_print_segment);
        let (before_wipe, during_wipe) = if let Some(segment) = wipe_segment {
            let feedrate = selected_wipe_feedrate(segment, role_based_wipe_speed, wipe_feedrate);
            retraction_split(RetractionSplitInput {
                length,
                retract_before_wipe,
                retract_feedrate,
                wipe_feedrate: feedrate,
                wipe_distance,
                segment,
            })
        } else {
            (length, 0.0)
        };
        let mut gcode = String::new();
        if before_wipe > f64::EPSILON {
            gcode.push_str(&layer_change_retract_gcode(
                writer,
                LayerChangeRetractCommand {
                    use_firmware,
                    length: before_wipe,
                    feedrate: retract_feedrate,
                },
                retract_comment,
            ));
        }
        if let Some(segment) = wipe_segment {
            let feedrate = selected_wipe_feedrate(segment, role_based_wipe_speed, wipe_feedrate);
            let wipe_gcode =
                Self::wipe_gcode(writer, segment, wipe_distance, during_wipe, feedrate);
            gcode.push_str(&wipe_gcode);
        }
        let z_restore = if z_hop > 0.0
            && lift_enforce.allows(current_layer_is_first, previous_non_gap_fill_role)
        {
            let z_restore = writer.current_position().2;
            let raised_z = z_restore + z_hop;
            match self.z_hop_lift {
                ZHopLiftMode::Normal => {
                    gcode.push_str(&writer.travel_to_z_with_comment(
                        raised_z,
                        z_feedrate,
                        z_lift_comment,
                    ));
                }
                ZHopLiftMode::Auto { radians } | ZHopLiftMode::Slope { radians } => {
                    self.pending_travel_lift =
                        Some(slope_lift_move(writer, target, z_hop, radians, raised_z));
                }
                ZHopLiftMode::Spiral { radians } => {
                    self.pending_travel_lift = Some(spiral_lift_move(
                        writer,
                        target,
                        radians,
                        self.resolution,
                        raised_z,
                    ));
                }
            }
            Some(z_restore)
        } else {
            None
        };
        self.pending_unretract = true;
        self.pending_z_restore = z_restore;
        self.previous_print_segment = None;
        gcode
    }

    pub(crate) fn clear_z_restore_after_layer_z_move(&mut self) {
        self.pending_z_restore = None;
        self.pending_travel_lift = None;
    }

    pub(crate) fn unretract_before_print(
        &mut self,
        command: TravelUnretractCommand<'_>,
    ) -> (String, f64) {
        if !self.pending_unretract || command.kind != ToolpathMoveKind::Print {
            return (String::new(), 0.0);
        }

        let writer = command.writer;
        let mut gcode = layer_change_z_restore_gcode(
            writer,
            &mut self.pending_z_restore,
            command.z_feedrate,
            command.z_restore_comment,
        );
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
        self.pending_unretract = false;
        self.pending_z_restore = None;
        (gcode, e_offset_delta)
    }

    pub(crate) fn observe_completed_move(
        &mut self,
        kind: ToolpathMoveKind,
        start: Point2,
        end: Point2,
        feedrate: f64,
    ) {
        if kind == ToolpathMoveKind::Print {
            if distance(start, end) > f64::EPSILON {
                self.previous_print_segment = Some(PreviousPrintSegment {
                    start,
                    end,
                    feedrate,
                });
            }
            self.has_printed_move = true;
        } else {
            self.previous_print_segment = None;
        }
    }

    fn wipe_gcode(
        writer: &mut GCodeWriter,
        segment: PreviousPrintSegment,
        wipe_distance: f64,
        during_wipe: f64,
        feedrate: f64,
    ) -> String {
        let Some((target, _)) = wipe_target(segment, wipe_distance) else {
            return String::new();
        };
        writer.extrude_to_xy_with_feedrate_and_comment(
            target,
            -during_wipe,
            feedrate,
            Some("wipe and retract"),
        )
    }

    fn should_retract(&self, command: &TravelRetractionCommand<'_>) -> bool {
        let current_position = command.writer.current_position();
        let dx = current_position.0 - command.target.x();
        let dy = current_position.1 - command.target.y();

        let should_retract = command.kind == ToolpathMoveKind::Travel
            && self.has_printed_move
            && !command.pending_layer_change_unretract
            && !self.pending_unretract
            && command.travel_retraction_enabled
            && dx.hypot(dy) >= command.minimum_travel;

        should_retract && !reduce_infill_retraction_applies(command)
    }
}

fn wipe_target(segment: PreviousPrintSegment, wipe_distance: f64) -> Option<(Point2, f64)> {
    let length = distance(segment.start, segment.end);
    if length <= f64::EPSILON || wipe_distance <= 0.0 {
        return None;
    }
    let used = wipe_distance.min(length);
    let ratio = used / length;
    Some((
        Point2::new(
            segment.end.x() + (segment.start.x() - segment.end.x()) * ratio,
            segment.end.y() + (segment.start.y() - segment.end.y()) * ratio,
        ),
        used,
    ))
}

fn selected_wipe_feedrate(
    segment: PreviousPrintSegment,
    role_based_wipe_speed: bool,
    wipe_feedrate: f64,
) -> f64 {
    if role_based_wipe_speed {
        segment.feedrate
    } else {
        wipe_feedrate
    }
    .max(600.0)
}

fn retraction_split(input: RetractionSplitInput) -> (f64, f64) {
    let RetractionSplitInput {
        length,
        retract_before_wipe,
        retract_feedrate,
        wipe_feedrate,
        wipe_distance,
        segment,
    } = input;
    let base_before = length * retract_before_wipe;
    let remaining = length - base_before;
    let available = wipe_distance.min(distance(segment.start, segment.end));
    let max_during = retract_feedrate * available / wipe_feedrate;
    let during = remaining.min(max_during);
    (base_before + (remaining - during), during)
}

fn reduce_infill_retraction_applies(command: &TravelRetractionCommand<'_>) -> bool {
    command.reduce_infill_retraction
        && command.sparse_infill_density_positive
        && command
            .previous_non_gap_fill_role
            .is_some_and(reduce_infill_retraction_role)
        && reduce_infill_retraction_role(command.role)
}

fn reduce_infill_retraction_role(role: PrintPathRole) -> bool {
    matches!(
        role,
        PrintPathRole::SparseInfill
            | PrintPathRole::SolidInfill
            | PrintPathRole::TopSolidInfill
            | PrintPathRole::BottomSurface
            | PrintPathRole::Bridge
            | PrintPathRole::InternalBridge
    )
}
