//! Pre-heating usage blocks and the second-pass injection of `GCodeProcessor`
//! (`GCodeProcessor.cpp:1607-1943,1960-2141`) on the static filament path.
//! Physical nozzle ownership is borrowed from the retained grouping result.

mod injector;

use std::collections::BTreeMap;

use crate::ProjectSettings;
use crate::gcode_processor::MoveVertex;
use crate::project_slice::gcode_emit::tool_ordering::NozzleGroup;

/// `(unsigned)-1`: a machine G-code tag that was not seen.
const SENTINEL: u32 = u32::MAX;

/// `ExtruderPreHeating::FilamentUsageBlock` (`GCodeProcessor.hpp:490-498`).
#[derive(Clone, Copy)]
struct FilamentUsage {
    filament: i32,
    extruder: i32,
    nozzle: i32,
    lower: u32,
    upper: u32,
}

/// The usage blocks of `run_post_process`, in output line ids after `remap_gcode_id`.
struct UsageBlocks {
    filaments: Vec<FilamentUsage>,
    skippable: Vec<(u32, u32)>,
    start_end: u32,
    end_start: u32,
}

/// `TimeProcessor::InsertedLinesMap` restricted to the injector line types.
type InsertedLines = BTreeMap<u32, Vec<String>>;

/// `run_second_pass_injection`: splices the injector lines into `gcode` after the output lines
/// they are keyed on and shifts `moves` past them (`handle_offsets_of_second_process`).
pub(super) fn inject(
    lines: &[String],
    line_map: &[u32],
    settings: &ProjectSettings,
    (nozzle_group_result, nozzle_counts): (&[NozzleGroup], &[i64]),
    (gcode, moves): (&mut Vec<u8>, &mut [MoveVertex]),
) {
    let blocks = UsageBlocks::build(lines, settings, nozzle_group_result).remap(line_map);
    // `:2005-2014`: the injector reads a float prefix sum of the per-move times.
    let mut elapsed = 0.0_f32;
    let times = moves
        .iter()
        .map(|vertex| {
            elapsed += vertex.time;
            elapsed
        })
        .collect::<Vec<_>>();
    let inserted = injector::Injector::new(
        settings,
        (nozzle_group_result, nozzle_counts),
        moves,
        &times,
        &blocks,
    )
    .run();
    if inserted.is_empty() {
        return;
    }
    splice(gcode, &inserted);
    let mut keys = inserted.iter().peekable();
    let mut offset = 0;
    for vertex in moves {
        while let Some((_, added)) = keys.next_if(|(key, _)| **key < vertex.gcode_id) {
            offset += added.len() as u32;
        }
        vertex.gcode_id += offset;
    }
}

/// `:2087-2108`: one cursor over the keys, advanced only when a line id equals it.
fn splice(gcode: &mut Vec<u8>, inserted: &InsertedLines) {
    let added = inserted.values().flatten().map(String::len).sum::<usize>();
    let mut output = Vec::with_capacity(gcode.len() + added);
    let mut keys = inserted.iter().peekable();
    for (index, line) in gcode.split_inclusive(|byte| *byte == b'\n').enumerate() {
        output.extend_from_slice(line);
        if let Some((_, lines)) = keys.next_if(|(key, _)| **key as usize == index + 1) {
            lines
                .iter()
                .for_each(|line| output.extend_from_slice(line.as_bytes()));
        }
    }
    *gcode = output;
}

impl UsageBlocks {
    /// `build_usage_blocks` over every raw processor input line (`:1689-1771,1824-1833`) and
    /// the close-out of the last filament block (`:1885-1886`).
    fn build(
        lines: &[String],
        settings: &ProjectSettings,
        nozzle_group_result: &[NozzleGroup],
    ) -> Self {
        let mut blocks = Self {
            filaments: Vec::new(),
            skippable: Vec::new(),
            start_end: SENTINEL,
            end_start: SENTINEL,
        };
        for (index, line) in lines.iter().enumerate() {
            blocks.line(line, index as u32 + 1, settings, nozzle_group_result);
        }
        if let Some(last) = blocks.filaments.last_mut() {
            last.upper = blocks.end_start;
        }
        blocks
    }

    fn line(
        &mut self,
        line: &str,
        id: u32,
        settings: &ProjectSettings,
        nozzle_group_result: &[NozzleGroup],
    ) {
        match line.strip_prefix(';') {
            Some(" MACHINE_START_GCODE_END") => return self.start_end = id,
            Some(" MACHINE_END_GCODE_START") => return self.end_start = id,
            Some(" SKIPPABLE_START") => self.skippable.push((id, 0)),
            Some(" SKIPPABLE_END") => {
                if let Some(last) = self.skippable.last_mut() {
                    last.1 = id;
                }
            }
            _ => {}
        }
        let command = line.trim_start_matches([' ', '\t']);
        let selection = if let Some(rest) = command.strip_prefix('T') {
            Some(rest)
        } else if let Some(rest) = command.strip_prefix(";VT") {
            Some(rest)
        } else if command.starts_with("M1020") {
            line.find('S').map(|position| &line[position + 1..])
        } else {
            None
        };
        if let Some(selection) = selection.and_then(filament_change) {
            self.filament_change(selection, id, settings, nozzle_group_result);
        }
    }

    /// `handle_filament_change` (`:1638-1656`).
    fn filament_change(
        &mut self,
        (filament, nozzle): (i32, i32),
        id: u32,
        settings: &ProjectSettings,
        nozzle_group_result: &[NozzleGroup],
    ) {
        if self.start_end == SENTINEL || self.end_start != SENTINEL && id > self.end_start {
            return;
        }
        if let Some(last) = self.filaments.last_mut() {
            last.upper = id;
        }
        let nozzle = if nozzle == -1 {
            super::super::motion::tool_selection::nozzle(settings, filament as usize) as i32
        } else {
            nozzle
        };
        self.filaments.push(FilamentUsage {
            filament,
            extruder: extruder_of(nozzle, nozzle_group_result),
            nozzle,
            lower: id,
            upper: SENTINEL,
        });
    }

    /// `remap_gcode_id` (`:1151-1160`) applied at `:1925-1943`.
    fn remap(mut self, line_map: &[u32]) -> Self {
        let remap = |id: &mut u32| {
            if let Some(output) = line_map.get(*id as usize) {
                *id = *output;
            }
        };
        for block in &mut self.filaments {
            remap(&mut block.lower);
            remap(&mut block.upper);
        }
        for (start, end) in &mut self.skippable {
            remap(start);
            remap(end);
        }
        remap(&mut self.start_end);
        remap(&mut self.end_start);
        self
    }
}

/// `get_valid_extruder_id` (`GCodeProcessor.cpp:2217-2223`) over the retained grouping.
fn extruder_of(nozzle: i32, nozzle_group_result: &[NozzleGroup]) -> i32 {
    nozzle_group_result
        .get(nozzle as usize)
        .map_or(0, |group| group.extruder as i32)
}

/// The `istringstream` reads after `T`, `;VT` or `M1020 … S`: a filament id in `0..255`, then
/// the integer after the first later `H`, or -1 without one.
fn filament_change(rest: &str) -> Option<(i32, i32)> {
    let (Ok(filament), rest) = stream_int(rest)? else {
        return None;
    };
    if !(0..255).contains(&filament) {
        return None;
    }
    // C++11 stores 0 when no integer follows `H`, and the clamped value on overflow.
    let nozzle = rest.split_once('H').map_or(-1, |(_, after)| {
        stream_int(after).map_or(0, |(id, _)| id.unwrap_or_else(|clamped| clamped))
    });
    Some((filament, nozzle))
}

/// `std::istream >> int`: skips whitespace and reads an optional sign and digits. An
/// out-of-range value fails the extraction and stores the clamped value (`Err`).
fn stream_int(text: &str) -> Option<(Result<i32, i32>, &str)> {
    let text = text.trim_start_matches(|c: char| c.is_ascii_whitespace() || c == '\x0b');
    let sign = usize::from(text.starts_with(['+', '-']));
    let digits = text[sign..].bytes().take_while(u8::is_ascii_digit).count();
    if digits == 0 {
        return None;
    }
    let end = sign + digits;
    let value = text[..end].parse().map_err(|_| {
        if text.starts_with('-') {
            i32::MIN
        } else {
            i32::MAX
        }
    });
    Some((value, &text[end..]))
}
