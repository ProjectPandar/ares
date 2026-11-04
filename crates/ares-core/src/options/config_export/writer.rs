use crate::{OrcaBool, SliceError};

use super::{
    collector::{ConfigEntry, collect_config_entries},
    transform::transformed_for_export,
};
use crate::options::project_config_views::ProjectConfigViews;
use crate::project::raw_settings::{ProjectSettingsRaw, RawConfigValue};
/// CLI-oracle export state the interactive `Print::apply` passes never
/// observe (see `transform::apply_cli_oracle_state`).
#[derive(Default)]
pub(super) struct ExportOverrides {
    /// Raw preset `enable_prime_tower` (None when the raw settings lack
    /// the key — keep the resolved value).
    pub(super) enable_prime_tower: Option<OrcaBool>,
    /// Raw preset `filament_map` token (joined `;` list).
    pub(super) raw_filament_map_token: Option<String>,
    #[allow(dead_code)]
    pub(super) is_bbl: bool,
}

impl ExportOverrides {
    /// Bookkeeping lookups read the raw map directly and render with the
    /// generic (non-Floats) token — no export-echo serialization, so no
    /// error path and no silent skips.
    fn from_raw(raw_settings: &ProjectSettingsRaw, is_bbl: bool) -> Self {
        let enable_prime_tower = raw_settings
            .get("enable_prime_tower")
            .and_then(RawConfigValue::token)
            .and_then(|token| match token.as_str() {
                "1" | "true" => Some(OrcaBool(true)),
                "0" | "false" => Some(OrcaBool(false)),
                _ => None,
            });
        let raw_filament_map_token = raw_settings
            .get("filament_map")
            .and_then(RawConfigValue::token);
        Self {
            enable_prime_tower,
            raw_filament_map_token,
            is_bbl,
        }
    }
}

const START: &[u8] = b"; CONFIG_BLOCK_START\n";
const END: &[u8] = b"; CONFIG_BLOCK_END\n\n";
pub(crate) fn write_config_block(
    views: &ProjectConfigViews,
    raw_settings: &ProjectSettingsRaw,
    plate_index: usize,
    compat_temperatures: Option<(i32, i32)>,
    output: &mut Vec<u8>,
) -> Result<(), SliceError> {
    let transformed = transformed_for_export(
        &views.full,
        &ExportOverrides::from_raw(raw_settings, is_bbl_printer(views)),
    )?;
    let entries = collect_config_entries(&transformed).map_err(config_error)?;
    let mut scratch = Vec::new();
    scratch.extend_from_slice(START);
    super::full_config::write_entries(
        &transformed,
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
        let area = entries
            .iter()
            .find(|entry| entry.key == "printable_area")
            .map(|entry| entry.token.as_str())
            .unwrap_or_default();
        append_line(output, "bed_shape", area);
        if let Some((_, nozzle)) = compat_temperatures {
            append_line(output, "first_layer_temperature", &nozzle.to_string());
        }
        if let Some(height) = entries
            .iter()
            .find(|entry| entry.key == "initial_layer_print_height")
            .and_then(|entry| entry.token.parse::<f64>().ok())
        {
            append_line(output, "first_layer_height", &format!("{height:.3}"));
        }
    } else if let Some((_, nozzle)) = compat_temperatures {
        append_line(output, "first_layer_temperature", &nozzle.to_string());
    }
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
