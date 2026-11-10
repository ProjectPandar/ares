//! `GCodeProcessor::PreCoolingInjector` on the filament path
//! (`GCodeProcessor.cpp:2150-2424,2429-2473`).

use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::iter::once;

use super::{FilamentUsage, InsertedLines, UsageBlocks, extruder_of};
use crate::gcode_processor::MoveVertex;
use crate::{Nullable, NullableFloats, OrcaInts, ProjectSettings};

/// `room_temperature` (`:2374`).
const ROOM_TEMPERATURE: f32 = 25.0;

/// `ExtruderFreeBlock` (`GCodeProcessor.hpp:964-976`) of `build_by_filament_blocks`: the
/// partial range is empty at `lower` and `ignore_cooling_before_tower` is set.
struct FreeBlock {
    extruder: i32,
    lower: u32,
    upper: u32,
    last_filament: i32,
    next_filament: i32,
    last_nozzle: i32,
}

/// One `add_M104_lines` call (`:2333-2370`).
struct Command<'a> {
    extruder: i32,
    temperature: i32,
    /// The next filament of an `M632 S<filament> W` … `M633` group around a heating line.
    skippable: Option<i32>,
    comment: &'a str,
}

pub(super) struct Injector<'a> {
    settings: &'a ProjectSettings,
    moves: &'a [MoveVertex],
    times: &'a [f32],
    skippable: &'a [(u32, u32)],
    free_blocks: Vec<FreeBlock>,
    inserted: InsertedLines,
}

impl<'a> Injector<'a> {
    /// `build_by_filament_blocks` (`:2429-2473`).
    pub(super) fn new(
        settings: &'a ProjectSettings,
        moves: &'a [MoveVertex],
        times: &'a [f32],
        blocks: &'a UsageBlocks,
    ) -> Self {
        let mut per_extruder = BTreeMap::<i32, Vec<FilamentUsage>>::new();
        for block in &blocks.filaments {
            per_extruder.entry(block.extruder).or_default().push(*block);
        }
        let frame = |lower, upper| FilamentUsage {
            filament: -1,
            extruder: -1,
            nozzle: -1,
            lower,
            upper,
        };
        let start = frame(0, blocks.start_end);
        let end = frame(blocks.end_start, u32::MAX);
        let mut free_blocks = Vec::new();
        for (extruder, usages) in per_extruder {
            let framed = once(start)
                .chain(usages)
                .chain(once(end))
                .collect::<Vec<_>>();
            free_blocks.extend(framed.windows(2).map(|pair| FreeBlock {
                extruder,
                lower: pair[0].upper,
                upper: pair[1].lower,
                last_filament: pair[0].filament,
                next_filament: pair[1].filament,
                last_nozzle: if pair[0].nozzle == -1 {
                    pair[1].nozzle
                } else {
                    pair[0].nozzle
                },
            }));
        }
        free_blocks.sort_by_key(|block| (block.lower, block.upper));
        Self {
            settings,
            moves,
            times,
            skippable: &blocks.skippable,
            free_blocks,
            inserted: InsertedLines::new(),
        }
    }

    /// `process_pre_cooling_and_heating` (`:2150-2193`): per extruder, every free block cools
    /// and all but the last also heat.
    pub(super) fn run(mut self) -> InsertedLines {
        let mut per_extruder = BTreeMap::<i32, Vec<FreeBlock>>::new();
        for block in std::mem::take(&mut self.free_blocks) {
            per_extruder.entry(block.extruder).or_default().push(block);
        }
        for blocks in per_extruder.into_values() {
            let count = blocks.len();
            for (index, block) in blocks.iter().enumerate() {
                let current = self.temperature(block.last_filament, 140);
                let target = self.temperature(block.next_filament, 0);
                self.inject(block, (current, target), index + 1 != count);
            }
        }
        self.inserted
    }

    /// `get_nozzle_temp` without the first-layer temperature or the pre-heat delta: no
    /// filament gives `missing`.
    fn temperature(&self, filament: i32, missing: i32) -> f32 {
        if filament == -1 {
            missing as f32
        } else {
            self.nozzle_temperature(filament as usize) as f32
        }
    }

    /// `m_filament_nozzle_temp` (`:3189-3192`): the first-layer temperature replaces 0.
    fn nozzle_temperature(&self, filament: usize) -> i32 {
        let print = &self.settings.filament.print;
        match get_at(&print.nozzle_temperature, filament) {
            0 => get_at(&print.nozzle_temperature_initial_layer, filament),
            temperature => temperature,
        }
    }

    /// `is_pre_cooling_valid` and `get_partial_free_cooling_thres` (`:2224-2236`): the
    /// temperature drop of a valid post-extrusion cooling.
    fn post_extrusion_drop(&self, filament: i32) -> Option<f32> {
        let filaments = self.settings.filament.gcode.filament_diameter.0.len();
        let index = usize::try_from(filament)
            .ok()
            .filter(|index| *index < filaments)?;
        let nozzle = self.nozzle_temperature(index);
        let pre_cooling = &self
            .settings
            .filament
            .gcode
            .filament_pre_cooling_temperature;
        match pre_cooling.get(index)? {
            Nullable::Value(value) if value.0 > 0 && value.0 < nozzle => {
                Some(nozzle as f32 - value.0 as f32)
            }
            _ => None,
        }
    }

    /// `inject_cooling_heating_command` (`:2217-2424`) with pre-cooling always on, so its
    /// heating-only branch is never taken.
    fn inject(&mut self, block: &FreeBlock, (mut current, target): (f32, f32), heating: bool) {
        let lower = self.position(block.lower);
        let upper = self.position(block.upper);
        // The empty partial range ends before `lower`, so a window from the first move returns.
        if lower == self.moves.len() || upper == 0 || lower == 0 {
            return;
        }
        let upper = upper - 1;
        let gap = self.times[upper] - self.times[lower - 1];
        // `inject_time_threshold` is 0; the partial gap adds 0.
        if lower >= upper || gap < 0.0 {
            return;
        }
        let extruder = extruder_of(self.settings, block.last_nozzle);
        let gcode = &self.settings.printer.gcode;
        let heating_rate = rate(&gcode.hotend_heating_rate, extruder);
        let cooling_rate = rate(&gcode.hotend_cooling_rate, extruder);
        let command = |temperature, skippable, comment| Command {
            extruder,
            temperature,
            skippable,
            comment,
        };
        if let Some(drop) = self.post_extrusion_drop(block.last_filament) {
            let partial_gap = 0.0_f32;
            let cooling = current.min(drop.min(partial_gap * cooling_rate));
            current = ROOM_TEMPERATURE.max(current - cooling);
            let comment = "Multi extruder pre cooling in post extrusion";
            self.add(block.lower, command(current as i32, None, comment));
        }
        if !heating {
            if target < current {
                let temperature = (ROOM_TEMPERATURE as i32).max(target as i32);
                self.add(
                    block.lower,
                    command(temperature, None, "Multi extruder pre cooling"),
                );
            }
            return;
        }
        let middle = ROOM_TEMPERATURE.max(
            (current * heating_rate + target * cooling_rate - gap * cooling_rate * heating_rate)
                / (cooling_rate + heating_rate),
        );
        let start = self.times[upper] - (target - middle) / heating_rate;
        // `std::upper_bound` with `start < time`.
        let after = lower
            + self.times[lower..=upper]
                .partition_point(|time| start.partial_cmp(time) != Some(Ordering::Less));
        if after == lower {
            return;
        }
        let heat = self.before_skippable(after - 1, lower);
        let cooling_time = self.times[heat] - self.times[lower];
        let drop = ((cooling_time * cooling_rate) as i32).min(current as i32);
        if drop == 0 {
            return;
        }
        let temperature = (ROOM_TEMPERATURE as i32).max(current as i32 - drop);
        self.add(
            block.lower,
            command(temperature, None, "Multi extruder pre cooling"),
        );
        let heating = command(
            target as i32,
            Some(block.next_filament),
            "Multi extruder pre heating",
        );
        self.add(self.moves[heat].gcode_id, heating);
    }

    /// `std::lower_bound` of the moves by `gcode_id`.
    fn position(&self, gcode_id: u32) -> usize {
        self.moves
            .partition_point(|vertex| vertex.gcode_id < gcode_id)
    }

    /// `adjust_iter` backwards (`:2265-2290`): moves the heating move before the SKIPPABLE
    /// range it sits in, without leaving `[lower, heat]`.
    fn before_skippable(&self, mut heat: usize, lower: usize) -> usize {
        while heat != lower {
            let id = self.moves[heat].gcode_id;
            let next = self.skippable.partition_point(|range| range.0 <= id);
            let Some(&(start, end)) = next.checked_sub(1).map(|index| &self.skippable[index])
            else {
                break;
            };
            if id > end || start == 0 {
                break;
            }
            let first =
                lower + self.moves[lower..heat].partition_point(|vertex| vertex.gcode_id < start);
            if first == lower {
                break;
            }
            heat = first - 1;
        }
        heat
    }

    /// `add_M104_lines` / `format_line_M104` (`:2333-2370`) for a printer without dynamic nozzle
    /// maps, multi-nozzle extruders or hotend-as-extruder addressing.
    fn add(&mut self, key: u32, command: Command<'_>) {
        let map = &self.settings.printer.gcode.physical_extruder_map.0;
        let physical = map
            .get(command.extruder as usize)
            .map_or(command.extruder, |id| id.0);
        let lines = self.inserted.entry(key).or_default();
        if let Some(filament) = command.skippable {
            lines.push(format!("M632 S{filament} W\n"));
        }
        lines.push("M400\n".to_owned());
        lines.push(format!(
            "M104 T{physical} S{} N0 ;{}\n",
            command.temperature, command.comment
        ));
        if command.skippable.is_some() {
            lines.push("M633\n".to_owned());
        }
    }
}

/// `ConfigOptionVector::get_at`.
fn get_at(values: &OrcaInts, index: usize) -> i32 {
    values.0.get(index).unwrap_or(&values.0[0]).0
}

/// `value_at_or_front(rates, extruder, 1.0)` narrowed to `float`; nil is NaN.
fn rate(rates: &NullableFloats, extruder: i32) -> f32 {
    let Some(first) = rates.0.first() else {
        return 1.0;
    };
    match rates.0.get(extruder as usize).unwrap_or(first) {
        Nullable::Value(value) => value.0 as f32,
        Nullable::Nil => f32::NAN,
    }
}
