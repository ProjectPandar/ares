use super::super::ProcessorConfig;
use super::super::estimate::options_z::{Corrector, MoveIds};
use super::super::motion::Trapezoid;
use super::MoveRecorder;
use super::dynamics::PreviewMachine;
use super::tags::{BBL_TAGS, COMPATIBLE_TAGS};
use crate::ExtrusionRole;
use crate::gcode_processor::{ChunkedMoves, MoveVertex};

#[derive(Clone, Copy, Eq, PartialEq)]
pub(in crate::project_slice::gcode_emit::processor) enum RecordMode {
    Full,
    CacheOnly,
    StatsOnly,
}

impl<'a> MoveRecorder<'a> {
    pub(in crate::project_slice::gcode_emit::processor) fn new(
        config: &'a ProcessorConfig<'_>,
        mode: RecordMode,
    ) -> Self {
        let settings = config.settings;
        let full = mode == RecordMode::Full;
        let count = settings.filament.gcode.filament_diameter.0.len();
        let filament_diameters = if full {
            settings
                .filament
                .gcode
                .filament_diameter
                .0
                .iter()
                .map(|diameter| diameter.0 as f32)
                .collect::<Vec<_>>()
        } else {
            Vec::new()
        };
        let offsets = &settings.project.print.extruder_offset.0;
        // `apply_config` (`:2052-2081`): `extruder_offset.get_at(filament_map[i] - 1)`.
        let extruder_offsets = if mode == RecordMode::StatsOnly {
            Vec::new()
        } else {
            (0..count)
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
                .collect()
        };
        Self {
            mode,
            mass_locator: full.then_some(config.mass_locator.as_ref()).flatten(),
            masses: if full {
                config
                    .mass_locator
                    .as_ref()
                    .map_or_else(Vec::new, |locator| locator.masses())
            } else {
                Vec::new()
            },
            filament_densities: &settings.filament.gcode.filament_density.0,
            total_travel_distance: 0.0,
            moves: if full {
                [MoveVertex::default()].into_iter().collect()
            } else {
                ChunkedMoves::default()
            },
            custom_gcode_per_print_z: Vec::new(),
            last_default_color: 0,
            color_change_volume: 0.0,
            color_change_volumes: Vec::new(),
            pending_option: None,
            options_z: if mode == RecordMode::StatsOnly {
                Corrector::empty()
            } else {
                Corrector::new()
            },
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
            extruder_temps: if full { vec![0.0; count] } else { Vec::new() },
            extruder_colors: if full {
                (0..count).map(|filament| filament as u8).collect()
            } else {
                Vec::new()
            },
            cp_color_current: 0,
            cp_color_counter: 0,
            extruder_offsets,
            filament_diameters,
            z_offset: settings.printer.remaining.z_offset.0 as f32,
            first_layer_height: settings.process.print.initial_layer_print_height.0.abs() as f32,
            spiral_vase_mode: settings.process.print.spiral_mode.0,
            last_line_id: 0,
            flavor: config.limits.gcode_flavor,
            machine: PreviewMachine::default(),
            pressure_advance: 0.0,
        }
    }

    pub(in crate::project_slice::gcode_emit::processor) const fn tracks_eligibility(&self) -> bool {
        !matches!(self.mode, RecordMode::StatsOnly)
    }

    pub(in crate::project_slice::gcode_emit::processor) fn next_vertex(&self) -> MoveIds {
        if self.mode == RecordMode::StatsOnly {
            return MoveIds::default();
        }
        self.options_z.next_vertex(self.moves.len())
    }

    pub(in crate::project_slice::gcode_emit::processor) fn queue_block(&mut self, ids: MoveIds) {
        if self.mode != RecordMode::StatsOnly {
            self.options_z.block(ids);
        }
    }

    pub(in crate::project_slice::gcode_emit::processor) fn queue_virtual(&mut self) {
        if self.mode != RecordMode::StatsOnly {
            self.options_z.virtual_block();
        }
    }

    pub(in crate::project_slice::gcode_emit::processor) fn apply_batch(
        &mut self,
        batch: &[(f64, Trapezoid)],
        eligible: &mut Vec<Option<bool>>,
    ) {
        match self.mode {
            RecordMode::Full => self.options_z.flush(&mut self.moves, batch, eligible),
            RecordMode::CacheOnly => self.options_z.flush_cache_only(batch, eligible),
            RecordMode::StatsOnly => eligible.clear(),
        }
    }

    /// `OptionsZCorrector::update` (`GCodeProcessor.hpp:693-705`).
    pub(super) fn correct_options_z(&mut self) {
        if self.mode == RecordMode::CacheOnly {
            self.options_z.update_cache_only();
            return;
        }
        if let Some(item_id) = self.pending_option.take() {
            let position = self.moves[self.moves.len() - 1].position;
            self.options_z.update(&mut self.moves, self.height);
            self.custom_gcode_per_print_z[item_id].print_z = f64::from(position[2]);
        }
    }

    pub(super) fn set_option(&mut self) {
        if self.mode == RecordMode::CacheOnly {
            self.options_z.set_cache_only();
            return;
        }
        self.pending_option = Some(self.custom_gcode_per_print_z.len() - 1);
        self.options_z.set(self.moves.len() - 1);
    }
}
