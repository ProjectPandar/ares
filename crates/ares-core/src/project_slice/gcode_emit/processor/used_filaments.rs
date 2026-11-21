//! `GCodeProcessor::UsedFilaments`, `process_filaments` and G1/VG1
//! volume accounting (`GCodeProcessor.cpp:2553-2573, 2647-2672, 4481-4482,
//! 5319-5325, 7668`).

use super::ProcessorConfig;
use super::filament_change::FilamentChange;

pub(super) struct UsedFilaments<'a> {
    diameters: &'a [crate::OrcaFloat],
    densities: &'a [crate::OrcaFloat],
    nozzle_volumes: Vec<f32>,
    remaining: [f32; 64],
    pending: [f64; 3],
    total: f64,
    bucket: usize,
    role: crate::ExtrusionRole,
    role_cache: f64,
    flushing: bool,
    virtual_flushing: bool,
    statistics: crate::PrintEstimatedStatistics,
}

impl<'a> UsedFilaments<'a> {
    pub(super) fn new(config: &ProcessorConfig<'a>) -> Self {
        let diameters = &config.settings.filament.gcode.filament_diameter.0;
        let count = diameters.len();
        Self {
            diameters,
            densities: &config.settings.filament.gcode.filament_density.0,
            nozzle_volumes: config
                .settings
                .printer
                .remaining
                .nozzle_volume
                .0
                .iter()
                .map(|value| match value {
                    crate::options::Nullable::Value(value) => value.0 as f32,
                    crate::options::Nullable::Nil => f32::NAN,
                })
                .collect(),
            remaining: [0.0; 64],
            pending: [0.0; 3],
            total: 0.0,
            bucket: 0,
            role: crate::ExtrusionRole::None,
            role_cache: 0.0,
            flushing: false,
            virtual_flushing: false,
            statistics: crate::PrintEstimatedStatistics {
                volumes_per_color_change: Vec::new(),
                used_filaments_per_role: std::collections::BTreeMap::new(),
                model_volumes_per_extruder: vec![0.0; count],
                support_volumes_per_extruder: vec![0.0; count],
                wipe_tower_volumes_per_extruder: vec![0.0; count],
                total_volumes_per_extruder: vec![0.0; count],
                flush_per_filament: vec![0.0; count],
                total_used_filament: 0.0,
                normal_time: 0.0,
                normal_prepare_time: 0.0,
            },
        }
    }

    pub(super) fn tags(&mut self, line: &str, filament: usize) {
        let Some((_, comment)) = line.split_once(';') else {
            return;
        };
        if let Some(role) = comment
            .strip_prefix("TYPE:")
            .or_else(|| comment.strip_prefix(" FEATURE: "))
        {
            // set_extrusion_role flushes before replacing the active role.
            self.process_role_cache(filament);
            self.role = crate::ExtrusionRole::from_name(role);
            self.bucket = match role {
                "Support" | "Support interface" | "Support transition" => 1,
                "Prime tower" => 2,
                _ => 0,
            };
        }
        // process_tags WIPE_TOWER_END, GCodeProcessor.cpp:4481-4482.
        if comment.starts_with(" WIPE_TOWER_END") {
            self.flush_wipe_tower(filament);
        }
        if comment.starts_with(" FLUSH_START") {
            self.flushing = true;
        }
        if comment.starts_with(" FLUSH_END") {
            self.flushing = false;
        }
        if comment.starts_with(" VFLUSH_START") {
            self.virtual_flushing = true;
        }
        if comment.starts_with(" VFLUSH_END") {
            self.virtual_flushing = false;
        }
    }

    pub(super) fn tool_change(&mut self, physical: usize, filament: usize) {
        // ToolChange flushes model/support/total, not the wipe-tower cache.
        self.flush_model_support_total(filament);
        self.remaining[physical] = self.nozzle_volumes[physical];
    }

    /// `ElegooGCodeProcessorHelper.cpp:170–189`: purge volume uses the original L word.
    pub(super) fn elegoo_purge(
        &mut self,
        physical: usize,
        previous: Option<usize>,
        purge: &super::elegoo::Purge,
    ) {
        let radius = 0.5_f32 * self.diameters[purge.filament].0 as f32;
        let volume = (std::f32::consts::PI * (radius * radius)) * purge.length;
        let nozzle_volume = self.nozzle_volumes.get(physical).copied().unwrap_or(0.0);
        if volume >= nozzle_volume {
            if let Some(previous) = previous.filter(|&id| id != purge.filament) {
                self.update_flush(previous, nozzle_volume);
            }
            self.update_flush(purge.filament, volume - nozzle_volume);
            self.remaining[physical] = 0.0;
        } else {
            self.update_flush(purge.filament, volume);
            self.remaining[physical] -= volume;
        }
    }

    pub(super) fn motion(
        &mut self,
        delta: [f64; 4],
        virtual_move: bool,
        wiping: bool,
        filaments: &FilamentChange<'_>,
    ) {
        if wiping || delta[3] <= 0.0 {
            return;
        }
        let (physical, active, previous) = filaments.selection();
        let radius = 0.5_f32 * self.diameters[active].0 as f32;
        let area = std::f32::consts::PI * (radius * radius);
        let volume = (f64::from(area) * delta[3]) as f32;
        let xy = delta[0] != 0.0 || delta[1] != 0.0;
        if !virtual_move && xy {
            self.pending[self.bucket] += f64::from(volume);
            self.total += f64::from(volume);
            self.role_cache += f64::from(volume);
            return;
        }
        let flushing = if virtual_move {
            self.virtual_flushing
        } else {
            self.flushing
        };
        if !flushing || xy || delta[2] != 0.0 {
            return;
        }
        let remaining = self.remaining[physical];
        if remaining > volume {
            self.update_flush(previous, volume);
            self.remaining[physical] -= volume;
        } else {
            self.update_flush(previous, remaining);
            self.update_flush(active, volume - remaining);
            self.remaining[physical] = 0.0;
        }
    }

    fn update_flush(&mut self, filament: usize, volume: f32) {
        if volume == 0.0 {
            return;
        }
        self.statistics.flush_per_filament[filament] += f64::from(volume);
        self.statistics.total_volumes_per_extruder[filament] += f64::from(volume);
    }

    fn flush_model_support_total(&mut self, filament: usize) {
        self.statistics.model_volumes_per_extruder[filament] += self.pending[0];
        self.statistics.support_volumes_per_extruder[filament] += self.pending[1];
        self.statistics.total_volumes_per_extruder[filament] += self.total;
        self.pending[0] = 0.0;
        self.pending[1] = 0.0;
        self.total = 0.0;
    }

    fn flush_wipe_tower(&mut self, filament: usize) {
        self.statistics.wipe_tower_volumes_per_extruder[filament] += self.pending[2];
        self.pending[2] = 0.0;
    }

    /// `UsedFilaments::process_role_cache` (`GCodeProcessor.cpp:2647-2665`).
    fn process_role_cache(&mut self, filament: usize) {
        if self.role_cache == 0.0 {
            return;
        }
        let radius = 0.5 * f64::from(self.diameters[filament].0 as f32);
        let area = std::f64::consts::PI * (radius * radius);
        // apply_config uses ConfigOptionVector::get_at (GCodeProcessor.cpp:2206,
        // Config.hpp:676-682): scalar density applies to later filaments.
        let density =
            f64::from(self.densities.get(filament).unwrap_or(&self.densities[0]).0 as f32);
        let filament_usage = (
            self.role_cache / area * 0.001,
            self.role_cache * density * 0.001,
        );
        self.statistics
            .used_filaments_per_role
            .entry(self.role)
            .and_modify(|usage| {
                usage.0 += filament_usage.0;
                usage.1 += filament_usage.1;
            })
            .or_insert(filament_usage);
        self.role_cache = 0.0;
    }

    pub(super) fn finish(mut self, filament: usize) -> crate::PrintEstimatedStatistics {
        self.process_role_cache(filament);
        self.flush_model_support_total(filament);
        self.flush_wipe_tower(filament);
        self.statistics.total_used_filament = self
            .statistics
            .total_volumes_per_extruder
            .iter()
            .zip(self.diameters)
            .map(|(volume, diameter)| {
                let diameter = f64::from(diameter.0 as f32);
                volume / (std::f64::consts::PI * (0.5 * diameter).powi(2))
            })
            .sum();
        self.statistics
    }
}
