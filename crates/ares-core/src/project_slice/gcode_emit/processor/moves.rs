//! `GCodeProcessorResult::moves` recording (`GCodeProcessor.cpp:3113-3330`
//! tags, `:3812-4234` `process_G1`, `:5004-5081` M104/M106/M107/M109,
//! `:5540-5605` filament change, `:5630-5718` `store_move_vertex`).

mod actual_speed;
mod extrusion;
pub(super) mod tags;

use super::ProcessorConfig;
use super::motion::Trapezoid;
use super::motion_util::{MMMIN_TO_MMSEC, word};
use crate::ExtrusionRole;
use crate::gcode_processor::{CustomGCodeItem, EMoveType, GCodeProcessorResult, MoveVertex};
use tags::{BBL_TAGS, COMPATIBLE_TAGS, ParsedLine, Tags};

const DEFAULT_TOOLPATH_WIDTH: f32 = 0.4;
const DEFAULT_TOOLPATH_HEIGHT: f32 = 0.2;
const WIPE_WIDTH: f32 = 0.05;
const WIPE_HEIGHT: f32 = 0.05;

/// One `process_G1` displacement (a linear move or one arc segment).
#[derive(Clone, Copy)]
pub(super) struct RecordedMotion {
    /// 1-based processor input line.
    pub(super) source: usize,
    /// Absolute XYZE before and after the move.
    pub(super) start: [f64; 4],
    pub(super) end: [f64; 4],
    pub(super) wiping: bool,
    /// `G1DiscretizationOrigin::G1`; arc segments are `G2G3`.
    pub(super) linear: bool,
    /// `SeamsDetector` closing midpoint associated with this move.
    pub(super) seam: Option<[f32; 3]>,
}

pub(super) struct MoveRecorder {
    moves: Vec<MoveVertex>,
    custom_gcode_per_print_z: Vec<CustomGCodeItem>,
    last_default_color: usize,
    color_change_volume: f64,
    color_change_volumes: Vec<f64>,
    pending_option: Option<(usize, usize)>,
    tags: &'static Tags,
    role: ExtrusionRole,
    processing_start_custom_gcode: bool,
    layer_id: u32,
    forced_width: f32,
    forced_height: f32,
    width: f32,
    height: f32,
    mm3_per_mm: f32,
    travel_dist: f32,
    extruded_last_z: f32,
    fan_speed: f32,
    feedrate: f32,
    filament: usize,
    extruder_temps: Vec<f32>,
    extruder_colors: Vec<u8>,
    cp_color_current: u8,
    cp_color_counter: u8,
    extruder_offsets: Vec<[f32; 2]>,
    filament_diameters: Vec<f32>,
    z_offset: f32,
    first_layer_height: f32,
    spiral_vase_mode: bool,
    /// `m_last_line_id`, in processor input lines.
    last_line_id: u32,
}

impl MoveRecorder {
    pub(super) fn new(config: &ProcessorConfig<'_>) -> Self {
        let settings = config.settings;
        let filament_diameters = settings
            .filament
            .gcode
            .filament_diameter
            .0
            .iter()
            .map(|diameter| diameter.0 as f32)
            .collect::<Vec<_>>();
        let count = filament_diameters.len();
        let offsets = &settings.project.print.extruder_offset.0;
        // `apply_config` (`:2052-2081`): `extruder_offset.get_at(filament_map[i] - 1)`.
        let extruder_offsets = (0..count)
            .map(|filament| {
                let slot = settings
                    .project
                    .gcode
                    .filament_map
                    .0
                    .get(filament)
                    .map_or(0, |slot| (slot.0 - 1).max(0) as usize);
                offsets
                    .get(slot)
                    .or(offsets.first())
                    .map_or([0.0; 2], |offset| [offset.x as f32, offset.y as f32])
            })
            .collect();
        Self {
            moves: vec![MoveVertex::default()],
            custom_gcode_per_print_z: Vec::new(),
            last_default_color: 0,
            color_change_volume: 0.0,
            color_change_volumes: Vec::new(),
            pending_option: None,
            tags: if config.limits.bbl_printer {
                &BBL_TAGS
            } else {
                &COMPATIBLE_TAGS
            },
            role: ExtrusionRole::None,
            processing_start_custom_gcode: false,
            layer_id: 0,
            forced_width: 0.0,
            forced_height: 0.0,
            width: 0.0,
            height: 0.0,
            mm3_per_mm: 0.0,
            travel_dist: 0.0,
            extruded_last_z: 0.0,
            fan_speed: 0.0,
            feedrate: 0.0,
            filament: 0,
            extruder_temps: vec![0.0; count],
            extruder_colors: (0..count).map(|filament| filament as u8).collect(),
            cp_color_current: 0,
            cp_color_counter: 0,
            extruder_offsets,
            filament_diameters,
            z_offset: settings.printer.remaining.z_offset.0 as f32,
            first_layer_height: settings.process.print.initial_layer_print_height.0.abs() as f32,
            spiral_vase_mode: settings.process.print.spiral_mode.0,
            last_line_id: 0,
        }
    }

    /// Tag comments and the non-motion state commands of one input line.
    /// `position` is the machine XYZE at the start of the line.
    pub(super) fn line(
        &mut self,
        source: usize,
        line: ParsedLine<'_>,
        g1_line_id: usize,
        position: [f64; 4],
    ) {
        if line.raw.starts_with(';') {
            if line.raw.len() > 2 {
                self.tag(source, line, g1_line_id, position);
            }
            return;
        }
        let code = line.raw.split(';').next().unwrap_or_default().trim();
        let mut words = code.split_whitespace();
        let filament = self.filament;
        match words.next() {
            Some("M104") => {
                if let Some(temperature) = word(code, 'S') {
                    self.extruder_temps[filament] = temperature as f32;
                }
            }
            Some("M109") => self.m109(code),
            Some("M106") => {
                if word(code, 'P').is_none_or(|fan| fan as f32 == 1.0) {
                    self.fan_speed =
                        word(code, 'S').map_or(100.0, |speed| (100.0_f32 / 255.0) * speed as f32);
                }
            }
            Some("M107") => self.fan_speed = 0.0,
            _ => {}
        }
    }

    /// `process_M109` (`:5081-5097`).
    fn m109(&mut self, code: &str) {
        let filament = self.filament;
        if let Some(temperature) = word(code, 'R') {
            let slot = match word(code, 'T') {
                Some(tool) => self.extruder_temps.get_mut(tool as usize),
                None => self.extruder_temps.get_mut(filament),
            };
            if let Some(slot) = slot {
                *slot = temperature as f32;
            }
        } else if let Some(temperature) = word(code, 'S') {
            self.extruder_temps[filament] = temperature as f32;
        }
    }

    /// `process_filament_change` (`:5540-5605`) tool-change vertex; returns
    /// the move id its time block carries.
    pub(super) fn tool_change(
        &mut self,
        source: usize,
        filament: usize,
        position: [f64; 4],
    ) -> u32 {
        self.filament = filament;
        self.cp_color_current = self.extruder_colors[filament];
        self.store(source, EMoveType::ToolChange, position, 0.0);
        self.moves.len() as u32 - 1
    }

    /// `m_feedrate` after a line's `F` word: linear moves narrow the f32
    /// product directly; the first arc segment round-trips it through
    /// `internal_only_g1_line`'s `(double)feedrate / MMMIN_TO_MMSEC` (`:4703`).
    pub(super) fn set_feedrate(&mut self, feedrate: f64, arc: bool) {
        self.feedrate = if arc {
            let mmmin = f64::from(MMMIN_TO_MMSEC);
            (feedrate / mmmin * mmmin) as f32
        } else {
            feedrate as f32
        };
    }

    /// `process_G1` from the move-type classification to `store_move_vertex`
    /// (`:3829-3947`, `:4190-4234`); returns the time block's `move_id`.
    pub(super) fn motion(&mut self, motion: RecordedMotion) -> u32 {
        let RecordedMotion {
            source,
            start,
            end,
            wiping,
            linear,
            seam,
        } = motion;
        let delta: [f64; 4] = std::array::from_fn(|axis| end[axis] - start[axis]);
        let move_type = move_type(wiping, delta);
        let delta_xyz =
            (delta[0] * delta[0] + delta[1] * delta[1] + delta[2] * delta[2]).sqrt() as f32;
        self.travel_dist = delta_xyz;
        if move_type == EMoveType::Extrude {
            self.extrusion_geometry(delta, delta_xyz, end[2], linear);
            self.correct_options_z();
        }
        let move_id = self.moves.len() as u32;
        if let Some(seam) = seam {
            // `set_end_position(0.5f * (new_pos + first) + m_z_offset * UnitZ)`
            // then `store_move_vertex(Seam)` subtracts `m_z_offset` again.
            let z = f64::from(seam[2] + self.z_offset);
            let source = self.last_line_id as usize;
            self.store(
                source,
                EMoveType::Seam,
                [f64::from(seam[0]), f64::from(seam[1]), z, 0.0],
                delta[3],
            );
        }
        self.store(source, move_type, end, delta[3]);
        move_id
    }

    /// OptionsZCorrector::update (GCodeProcessor.hpp:693-705): relocate only
    /// the latest pending marker, preserving its original source line.
    fn correct_options_z(&mut self) {
        if let Some((move_id, item_id)) = self.pending_option.take() {
            let position = self.moves[self.moves.len() - 1].position;
            let mut marker = self.moves[move_id];
            marker.position = position;
            marker.height = self.height;
            self.moves.push(marker);
            self.moves.remove(move_id);
            self.custom_gcode_per_print_z[item_id].print_z = f64::from(position[2]);
        }
    }

    /// `store_move_vertex` (`:5630-5718`); `source` is the input line the
    /// vertex reports before the export remap.
    fn store(&mut self, source: usize, move_type: EMoveType, end: [f64; 4], delta_extruder: f64) {
        self.last_line_id = source as u32;
        let offset = self
            .extruder_offsets
            .get(self.filament)
            .copied()
            .unwrap_or_default();
        let z = if self.processing_start_custom_gcode {
            self.first_layer_height
        } else {
            (end[2] - f64::from(self.z_offset)) as f32
        };
        self.moves.push(MoveVertex {
            gcode_id: source as u32,
            move_type,
            extrusion_role: self.role,
            extruder_id: self.filament as u8,
            cp_color_id: self.cp_color_current,
            position: [end[0] as f32 + offset[0], end[1] as f32 + offset[1], z],
            delta_extruder: delta_extruder as f32,
            feedrate: self.feedrate,
            width: self.width,
            height: self.height,
            mm3_per_mm: self.mm3_per_mm,
            travel_dist: self.travel_dist,
            fan_speed: self.fan_speed,
            temperature: self.extruder_temps[self.filament],
            time: 0.0,
            layer_id: self.layer_id.max(1) - 1,
            internal_only: false,
        });
    }

    pub(super) fn apply_schedule(
        &mut self,
        block_moves: &[u32],
        times: &[f64],
        trapezoids: &[Trapezoid],
    ) {
        actual_speed::apply(&mut self.moves, block_moves, times, trapezoids);
    }

    /// `finalize` wipe sizes (`:2689-2695`); `gcode_id` moves from processor
    /// input lines onto the exported file's 1-based lines
    /// (`ExportLines::synchronize_moves`, `:951-960`).
    pub(super) fn finish(mut self, line_map: &[u32]) -> GCodeProcessorResult {
        self.flush_color_change_volume();
        for vertex in &mut self.moves {
            if vertex.move_type == EMoveType::Wipe {
                vertex.width = WIPE_WIDTH;
                vertex.height = WIPE_HEIGHT;
            }
            if vertex.gcode_id != 0 {
                // `m_last_line_id = m_line_id + 1` for the marker vertices
                // (`:5664-5666`).
                let marker = matches!(
                    vertex.move_type,
                    EMoveType::ColorChange | EMoveType::PausePrint | EMoveType::CustomGCode
                );
                vertex.gcode_id = line_map[vertex.gcode_id as usize] + u32::from(marker);
            }
        }
        GCodeProcessorResult {
            moves: self.moves,
            custom_gcode_per_print_z: self.custom_gcode_per_print_z,
            print_statistics: crate::PrintEstimatedStatistics {
                volumes_per_color_change: self.color_change_volumes,
                ..Default::default()
            },
            spiral_vase_mode: self.spiral_vase_mode,
            ..Default::default()
        }
    }
}

/// `process_G1::move_type` (`:3835-3851`).
fn move_type(wiping: bool, delta: [f64; 4]) -> EMoveType {
    let [x, y, z, e] = delta;
    if wiping {
        EMoveType::Wipe
    } else if e < 0.0 {
        if x != 0.0 || y != 0.0 || z != 0.0 {
            EMoveType::Travel
        } else {
            EMoveType::Retract
        }
    } else if e > 0.0 {
        if x == 0.0 && y == 0.0 {
            if z == 0.0 {
                EMoveType::Unretract
            } else {
                EMoveType::Travel
            }
        } else {
            EMoveType::Extrude
        }
    } else if x != 0.0 || y != 0.0 || z != 0.0 {
        EMoveType::Travel
    } else {
        EMoveType::Noop
    }
}
