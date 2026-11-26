use std::io::Write;

use crate::SliceError;

use super::{
    collector::{ConfigEntry, collect_config_entries},
    transform::scaled_flush_matrix,
    value::serialize_config_value,
};
use crate::options::project_config_views::ProjectConfigViews;
use crate::project::{ConfigComposer, raw_settings::ProjectSettingsRaw};

const START: &[u8] = b"; CONFIG_BLOCK_START\n";
const END: &[u8] = b"; CONFIG_BLOCK_END\n\n";
const BED_SHAPE_PREFIX: &[u8] = b"; bed_shape = ";
const FIRST_LAYER_HEIGHT_PREFIX: &[u8] = b"; first_layer_height = ";

pub(crate) fn write_config_block(
    views: &ProjectConfigViews,
    raw_settings: (&ProjectSettingsRaw, ConfigComposer),
    plate_index: usize,
    compat_temperatures: Option<(i32, i32)>,
    output: &mut Vec<u8>,
) -> Result<(), SliceError> {
    let scaled_flush = scaled_flush_matrix(&views.full)?;
    let mut entries = collect_config_entries(&views.full).map_err(config_error)?;
    if let Some(matrix) = scaled_flush {
        let serialized = serialize_config_value(&matrix).map_err(config_error)?;
        if let Ok(index) =
            entries.binary_search_by(|entry| entry.key.as_str().cmp("flush_volumes_matrix"))
        {
            entries[index].token = serialized.token;
            entries[index].is_nil = serialized.is_nil;
        }
    }
    let mut scratch = Vec::with_capacity(16 * 1024);
    scratch.extend_from_slice(START);
    super::full_config::write_entries(
        &views.full,
        plate_index,
        &entries,
        raw_settings,
        &mut scratch,
    )?;
    write_runtime_tail(
        compat_temperatures,
        is_bbl_printer(views),
        &entries,
        &mut scratch,
    );
    scratch.extend_from_slice(END);
    output.extend_from_slice(&scratch);
    Ok(())
}

/// Derives the final config block with `compat_temperatures` from a previously
/// rendered `write_config_block(..., compat_temperatures = None)` snapshot by
/// inserting only the two runtime temperature lines in the same positions as
/// `write_runtime_tail`.
pub(crate) fn config_block_with_compat_temperatures(
    base: &[u8],
    views: &ProjectConfigViews,
    (bed, nozzle): (i32, i32),
) -> Option<Vec<u8>> {
    let body = base.strip_suffix(END)?;
    let mut out = Vec::with_capacity(base.len() + 96);
    if is_bbl_printer(views) {
        out.extend_from_slice(body);
        append_i32_line(&mut out, "first_layer_bed_temperature", bed);
        append_i32_line(&mut out, "first_layer_temperature", nozzle);
        out.extend_from_slice(END);
        return Some(out);
    }
    let (before_height, height_line) = split_trailing_line(body, FIRST_LAYER_HEIGHT_PREFIX);
    let (before_bed_shape, bed_shape_line) = split_trailing_line(before_height, BED_SHAPE_PREFIX);
    if bed_shape_line.is_empty() {
        return None;
    }
    out.extend_from_slice(before_bed_shape);
    append_i32_line(&mut out, "first_layer_bed_temperature", bed);
    out.extend_from_slice(bed_shape_line);
    append_i32_line(&mut out, "first_layer_temperature", nozzle);
    out.extend_from_slice(height_line);
    out.extend_from_slice(END);
    Some(out)
}

fn split_trailing_line<'a>(bytes: &'a [u8], prefix: &[u8]) -> (&'a [u8], &'a [u8]) {
    let Some(without_newline) = bytes.strip_suffix(b"\n") else {
        return (bytes, &[]);
    };
    let line_start = without_newline
        .iter()
        .rposition(|&byte| byte == b'\n')
        .map_or(0, |pos| pos + 1);
    let last_line = &bytes[line_start..];
    if last_line.starts_with(prefix) {
        (&bytes[..line_start], last_line)
    } else {
        (bytes, &[])
    }
}

fn append_i32_line(output: &mut Vec<u8>, key: &str, value: i32) {
    let _ = writeln!(output, "; {key} = {value}");
}

fn is_bbl_printer(views: &ProjectConfigViews) -> bool {
    views
        .full
        .printer
        .remaining
        .printer_model
        .0
        .starts_with("Bambu Lab")
}

fn write_runtime_tail(
    compat_temperatures: Option<(i32, i32)>,
    bbl_printer: bool,
    entries: &[ConfigEntry],
    output: &mut Vec<u8>,
) {
    // GCode.cpp:3681–3687,4892–4894: both snapshots precede the object loop.
    // Early template config has no ToolOrdering context for these scalars.
    if let Some((bed, _)) = compat_temperatures {
        append_line(output, "first_layer_bed_temperature", &bed.to_string());
    }
    // Upstream appends compatible info after the config keys: the non-BBL
    // footer also mirrors the printable_area serialization as bed_shape and
    // re-formats the initial layer print height at %.3f
    // (`GCode.cpp:3554-3562`); the BBL block stops at the two temperature
    // lines (`GCode.cpp:2648-2657`).
    if !bbl_printer {
        let area = find_entry(entries, "printable_area").unwrap_or_default();
        append_line(output, "bed_shape", area);
        if let Some((_, nozzle)) = compat_temperatures {
            append_line(output, "first_layer_temperature", &nozzle.to_string());
        }
        if let Some(height) = find_entry(entries, "initial_layer_print_height")
            .and_then(|token| token.parse::<f64>().ok())
        {
            append_line(output, "first_layer_height", &format!("{height:.3}"));
        }
    } else if let Some((_, nozzle)) = compat_temperatures {
        append_line(output, "first_layer_temperature", &nozzle.to_string());
    }
}

fn find_entry<'a>(entries: &'a [ConfigEntry], key: &str) -> Option<&'a str> {
    entries
        .binary_search_by(|entry| entry.key.as_str().cmp(key))
        .ok()
        .map(|index| entries[index].token.as_str())
}

pub(super) fn append_line(output: &mut Vec<u8>, key: &str, token: &str) {
    output.extend_from_slice(b"; ");
    output.extend_from_slice(key.as_bytes());
    output.extend_from_slice(b" = ");
    output.extend_from_slice(token.as_bytes());
    output.push(b'\n');
}

fn config_error(error: impl std::fmt::Display) -> SliceError {
    invalid(error.to_string())
}

fn invalid(message: impl Into<String>) -> SliceError {
    SliceError::InvalidInput(message.into())
}
