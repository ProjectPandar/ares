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

/// The accumulated last extrusion path for wiping — upstream `Wipe::path`
/// (`GCode.hpp:59-71`, fed per extruded path at `GCode.cpp:5980-5990`,
/// reset after wiping at `:496`).
#[derive(Debug, Default)]
struct WipePath {
    points: Vec<Point2>,
    feedrate: f64,
}

/// Scaled-coordinate space of `libslic3r` points (1e6 units per mm).
const SCALE: f64 = 1_000_000.0;

/// `Line::length()`: `sqrt(dx² + dy²)` over scaled integer coordinates.
fn scaled_length(start: Point2, end: Point2) -> f64 {
    let dx = (end.x() - start.x()) * SCALE;
    let dy = (end.y() - start.y()) * SCALE;
    (dx * dx + dy * dy).sqrt()
}

impl WipePath {
    fn observe_print_move(&mut self, start: Point2, end: Point2, feedrate: f64) {
        if distance(start, end) <= f64::EPSILON {
            return;
        }
        match self.points.last() {
            Some(&last) if last == start => {
                // `GCode.cpp:5984-5988`: don't save duplicated points.
                if self.points.last() != Some(&end) {
                    self.points.push(end);
                }
                self.feedrate = feedrate;
            }
            _ => {
                // A new path begins (travel gap or fresh start): replace.
                self.points = vec![start, end];
                self.feedrate = feedrate;
            }
        }
    }

    /// `Wipe::wipe` `:450-453`: `[last_pos] + path.points[1..]` over the
    /// REVERSED path (the wipe walks backward from the current position;
    /// `m_wipe.path` stores the reversed emission order).
    fn with_current_position(&self, current: Point2) -> Vec<Point2> {
        let mut points: Vec<Point2> = self.points.iter().rev().copied().collect();
        if let Some(first) = points.first_mut() {
            *first = current;
        }
        points
    }

    fn length(points: &[Point2]) -> f64 {
        // `MultiPoint::length` sums `Line::length()` — `sqrt(dx²+dy²)` over
        // the SCALED INTEGER points (`Polyline.cpp`), not the compensated
        // `hypot` over millimetres.
        points
            .windows(2)
            .map(|pair| scaled_length(pair[0], pair[1]))
            .sum()
    }

    /// `Polyline::clip_end(len - wipe_dist)` (`:456`): walk from the END,
    /// interpolate the boundary point onto the INTEGER lattice (the C++
    /// `cast<coord_t>()` truncates toward zero) — the truncated endpoint
    /// shifts the segment ratio and flips 5th-decimal E knife edges.
    fn clip_to_length(points: &[Point2], keep: f64) -> Vec<Point2> {
        let total = Self::length(points);
        if total <= keep || points.len() < 2 {
            return points.to_vec();
        }
        // `clip_end(total - keep)` removes from the tail; equivalently keep
        // the leading `keep` millimetres, interpolating the final point with
        // the upstream backward formula truncated onto the lattice.
        let mut kept: Vec<Point2> = points.to_vec();
        let mut distance = total - keep;
        while distance > 0.0 {
            let Some(last_point) = kept.pop() else {
                return Vec::new();
            };
            if kept.is_empty() {
                return Vec::new();
            }
            let previous = *kept.last().expect("checked non-empty");
            let v = (
                (previous.x() - last_point.x()) * SCALE,
                (previous.y() - last_point.y()) * SCALE,
            );
            let lsqr = v.0 * v.0 + v.1 * v.1;
            if lsqr > distance * distance {
                let factor = distance / lsqr.sqrt();
                let x = last_point.x() * SCALE + v.0 * factor;
                let y = last_point.y() * SCALE + v.1 * factor;
                kept.push(Point2::new(
                    x.trunc() as i64 as f64 / SCALE,
                    y.trunc() as i64 as f64 / SCALE,
                ));
                break;
            }
            distance -= lsqr.sqrt();
        }
        kept
    }
}

struct PreviousPrintSegment {
    start: Point2,
    end: Point2,
    feedrate: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct RetractionSplitInput<'a> {
    length: f64,
    retract_before_wipe: f64,
    retract_feedrate: f64,
    wipe_feedrate: f64,
    wipe_distance: f64,
    path: &'a [Point2],
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
    wipe_path: Option<WipePath>,
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
            wipe_path: None,
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
        let wipe_path = (wipe && wipe_distance > 0.0 && !use_firmware)
            .then_some(())
            .and(self.wipe_path.take());
        let (before_wipe, during_wipe) = if let Some(path) = wipe_path.as_ref() {
            let feedrate = selected_wipe_feedrate(
                PreviousPrintSegment {
                    start: *path.points.first().unwrap_or(&Point2::new(0.0, 0.0)),
                    end: *path.points.last().unwrap_or(&Point2::new(0.0, 0.0)),
                    feedrate: path.feedrate,
                },
                role_based_wipe_speed,
                wipe_feedrate,
            );
            retraction_split(RetractionSplitInput {
                length,
                retract_before_wipe,
                retract_feedrate,
                wipe_feedrate: feedrate,
                wipe_distance,
                path: &path.points,
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
        if let Some(path) = wipe_path.as_ref() {
            let feedrate = selected_wipe_feedrate(
                PreviousPrintSegment {
                    start: *path.points.first().unwrap_or(&Point2::new(0.0, 0.0)),
                    end: *path.points.last().unwrap_or(&Point2::new(0.0, 0.0)),
                    feedrate: path.feedrate,
                },
                role_based_wipe_speed,
                wipe_feedrate,
            );
            let current = writer.current_position();
            let wipe_gcode = Self::wipe_gcode(
                writer,
                path,
                Point2::new(current.0, current.1),
                wipe_distance,
                during_wipe,
                feedrate,
            );
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
            // Upstream keeps the wipe path across travels (`Wipe::path` is
            // only reset by the wipe itself and the special sites,
            // `GCode.cpp:496/831/1107/1199/1388`); a print move after a gap
            // starts a fresh path (`:5980-5990` replaces per path).
            self.wipe_path
                .get_or_insert_with(WipePath::default)
                .observe_print_move(start, end, feedrate);
            self.has_printed_move = true;
        }
    }

    fn wipe_gcode(
        writer: &mut GCodeWriter,
        path: &WipePath,
        current: Point2,
        wipe_distance: f64,
        during_wipe: f64,
        feedrate: f64,
    ) -> String {
        // `Wipe::wipe` `:450-462`: substitute the current position for the
        // first point, clip to the trailing `wipe_dist`, and distribute
        // the during-wipe retraction proportionally per segment `:473-475`.
        let wipe_path = path.with_current_position(current);
        if wipe_path.len() < 2 {
            return String::new();
        }
        // Upstream works entirely in scaled units: `wipe_dist =
        // scale_(wipe_distance)` (`GCode.cpp:444-446`) and lengths over the
        // integer points.
        let mut wipe_dist = WipePath::length(&wipe_path).min(wipe_distance * SCALE);
        if wipe_dist <= f64::EPSILON {
            return String::new();
        }
        let clipped = WipePath::clip_to_length(&wipe_path, wipe_dist);
        let actual = WipePath::length(&clipped);
        if actual < wipe_dist {
            wipe_dist = actual.max(f64::EPSILON);
        }
        let mut gcode = String::new();
        for pair in clipped.windows(2) {
            let segment_length = scaled_length(pair[0], pair[1]);
            let delta_e = during_wipe * (segment_length / wipe_dist);
            gcode.push_str(&writer.extrude_to_xy_with_feedrate_and_comment(
                pair[1],
                -delta_e,
                feedrate,
                Some("wipe and retract"),
            ));
        }
        gcode
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
        path,
    } = input;
    let base_before = length * retract_before_wipe;
    let remaining = length - base_before;
    let available = wipe_distance.min(WipePath::length(path));
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
