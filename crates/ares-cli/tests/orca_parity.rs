//! OrcaSlicer parity suite: slices vendor printer profiles with both the
//! OrcaSlicer 2.4.2 CLI and Ares and compares the G-code with the KSR
//! semantic comparator.
//!
//! Environment-gated: runs only when `ARES_ORCA_BIN` (or the repository
//! wrapper `scripts/orca-parity.sh`) names a working OrcaSlicer CLI.

#[path = "orca_parity/presets.rs"]
mod presets;
#[path = "orca_parity/runner.rs"]
mod runner;
#[path = "orca_parity/smoke.rs"]
mod smoke;

#[path = "ksr_fdmtest_v4/semantic.rs"]
mod semantic;

use std::collections::BTreeMap;

use presets::VendorProfiles;
use runner::{CaseInputs, OrcaRunner};
use serde_json::{Map, Value};

use crate::runner::{OrcaRunner as Runner, ParityCase};

pub(crate) struct ParityOutcome {
    pub(crate) label: String,
    pub(crate) status: &'static str,
    pub(crate) detail: String,
}

pub(crate) fn compare_case(case: &ParityCase) -> ParityOutcome {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .expect("tokio runtime");
    let actual = runtime.block_on(async {
        ares_core::slice_project(
            &case.project,
            ares_core::GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
        )
        .await
    });
    match actual {
        Ok(actual) => match semantic::compare_ignoring_time(&case.reference, &actual) {
            Ok(()) => pass(&case.label),
            Err(difference) => divergence(&case.label, difference),
        },
        Err(error) => ares_error(&case.label, error.to_string()),
    }
}

pub(crate) fn pass(label: &str) -> ParityOutcome {
    ParityOutcome {
        label: label.to_owned(),
        status: "PASS",
        detail: String::new(),
    }
}

pub(crate) fn divergence(label: &str, difference: String) -> ParityOutcome {
    ParityOutcome {
        label: label.to_owned(),
        status: "DIVERGENT",
        detail: difference,
    }
}

pub(crate) fn ares_error(label: &str, error: String) -> ParityOutcome {
    ParityOutcome {
        label: label.to_owned(),
        status: "ARES_ERROR",
        detail: error,
    }
}

/// Baseline option overrides for the smoke matrix. The classic wall
/// generator is the ported perimeter engine; Arachne dispatch is not
/// implemented yet and is tracked as its own slice.
pub(crate) fn smoke_overrides() -> Map<String, Value> {
    let mut overrides = Map::new();
    overrides.insert(
        "wall_generator".to_owned(),
        Value::String("classic".to_owned()),
    );
    // Classic thin-wall gap detection is not ported yet (tracked as its own
    // slice); pin it off so the baseline exercises ported behavior.
    overrides.insert("detect_thin_wall".to_owned(), Value::String("0".to_owned()));
    // Some vendor profiles encode the whole bed boundary as the first
    // exclusion polygon. The standalone Orca CLI then arranges the smoke cube
    // outside the bed (`return -50`), unlike the GUI profile loader. Exclusions
    // do not affect generated movement after placement, so clear them for the
    // shared smoke fixture.
    overrides.insert(
        "bed_exclude_area".to_owned(),
        Value::Array(vec![Value::String("0x0".to_owned())]),
    );
    overrides.insert("post_process".to_owned(), Value::Array(Vec::new()));
    overrides
}

pub(crate) struct PrinterSelection {
    pub(crate) vendor: String,
    pub(crate) printer: String,
    pub(crate) process: String,
    pub(crate) filaments: Vec<String>,
}

/// Resolves the printer's default process and filament presets, applying the
/// vendor's nozzle-variant naming ("<base> 0.4 nozzle"/"<base> 0.4").
pub(crate) fn select_printer(
    profiles: &VendorProfiles,
    vendor: &str,
    printer: &str,
) -> Result<PrinterSelection, String> {
    let machine = profiles.machine(printer)?;
    let variant = machine
        .get("printer_variant")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_owned();

    let base_process = machine
        .get("default_print_profile")
        .and_then(Value::as_str)
        .map(str::to_owned);
    let process = match pick_preset(
        base_process.as_deref(),
        &variant,
        |name| profiles.process_exists(name),
        &format!("{vendor}/{printer} process"),
    ) {
        Ok(process) if profiles.process_is_compatible(&process, printer) => process,
        Ok(_) => profiles
            .compatible_process(printer)
            .ok_or_else(|| format!("{vendor}/{printer} process: no compatible preset"))?,
        Err(error) => profiles.compatible_process(printer).ok_or(error)?,
    };

    let mut filaments: Vec<String> = match machine.get("default_filament_profile") {
        Some(Value::Array(names)) => names.iter().filter_map(Value::as_str).collect(),
        Some(Value::String(name)) => vec![name.as_str()],
        _ => Vec::new(),
    }
    .into_iter()
    .filter(|name| {
        !name.is_empty()
            && profiles.filament_exists(name)
            && profiles.filament_is_compatible(name, printer)
    })
    .take(1)
    .map(ToOwned::to_owned)
    .collect();
    if filaments.is_empty() {
        filaments.push(
            profiles.compatible_filament(printer).ok_or_else(|| {
                format!("{vendor}/{printer}: no usable compatible filament preset")
            })?,
        );
    }

    Ok(PrinterSelection {
        vendor: vendor.to_owned(),
        printer: printer.to_owned(),
        process,
        filaments,
    })
}

fn pick_preset(
    base: Option<&str>,
    variant: &str,
    exists: impl Fn(&str) -> bool,
    what: &str,
) -> Result<String, String> {
    let Some(base) = base.filter(|base| !base.is_empty()) else {
        return Err(format!("{what}: machine names no default preset"));
    };
    for candidate in [
        Some(format!("{base} {variant} nozzle")),
        Some(format!("{base} {variant}")),
        (!variant.is_empty()).then(|| base.to_owned()),
        Some(base.to_owned()),
    ]
    .into_iter()
    .flatten()
    {
        if exists(&candidate) {
            return Ok(candidate);
        }
    }
    Err(format!("{what}: default preset {base:?} not found"))
}

pub(crate) fn build_selection_case(
    runner: &OrcaRunner,
    profiles: &VendorProfiles,
    selection: &PrinterSelection,
    model: &std::path::Path,
) -> Result<ParityCase, String> {
    let machine = profiles.machine(&selection.printer)?;
    let process = profiles.process(&selection.process)?;
    let filaments = selection
        .filaments
        .iter()
        .map(|name| profiles.filament(name))
        .collect::<Result<Vec<_>, _>>()?;
    let label = format!("{}/{}", selection.vendor, selection.printer);
    let overrides = smoke_case_overrides(&machine, &process);
    runner.build_case(
        &CaseInputs {
            label: &label,
            machine: &machine,
            process: &process,
            filaments: &filaments,
        },
        &overrides,
        model,
    )
}

fn smoke_case_overrides(
    machine: &Map<String, Value>,
    process: &Map<String, Value>,
) -> Map<String, Value> {
    let mut overrides = smoke_overrides();
    let relative_e = machine
        .get("use_relative_e_distances")
        .is_some_and(|value| value.as_str() == Some("1") || value.as_bool() == Some(true));
    let before = process
        .get("before_layer_change_gcode")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let layer = process
        .get("layer_change_gcode")
        .and_then(Value::as_str)
        .unwrap_or_default();
    if relative_e && !before.contains("G92 E0") && !layer.contains("G92 E0") {
        let separator = if before.is_empty() || before.ends_with('\n') {
            ""
        } else {
            "\n"
        };
        overrides.insert(
            "before_layer_change_gcode".to_owned(),
            Value::String(format!("{before}{separator}G92 E0")),
        );
    }
    if let Some(value) = clamp_vector(machine, "retraction_distances_when_cut", 10.0, 18.0) {
        overrides.insert("retraction_distances_when_cut".to_owned(), value);
    }
    if let Some(value) = clamp_vector(machine, "extruder_printable_height", 0.0, 1_000.0) {
        overrides.insert("extruder_printable_height".to_owned(), value);
    }
    if machine
        .get("use_firmware_retraction")
        .is_some_and(option_true)
        && process.get("wipe").is_some_and(option_true)
    {
        overrides.insert(
            "use_firmware_retraction".to_owned(),
            Value::String("0".to_owned()),
        );
    }
    let nozzle = machine
        .get("nozzle_diameter")
        .and_then(first_number)
        .unwrap_or(0.4);
    if process
        .get("bridge_line_width")
        .and_then(first_number)
        .is_some_and(|width| width > nozzle)
    {
        overrides.insert(
            "bridge_line_width".to_owned(),
            Value::String(nozzle.to_string()),
        );
    }
    if let Some(source) = machine.get("machine_start_gcode").and_then(Value::as_str) {
        let mut normalized = source.replace("[output_filename_format]", "[input_filename_base]");
        for placeholder in [
            "extruder_rotation_volume[0]",
            "mixing_stepper_rotation_volume[0]",
            "multi_zone_1_initial_layer[0]",
            "multi_zone_2_initial_layer[0]",
            "multi_zone_3_initial_layer[0]",
        ] {
            normalized = normalized.replace(&format!("{{{placeholder}}}"), "0");
        }
        if normalized != source {
            overrides.insert("machine_start_gcode".to_owned(), Value::String(normalized));
        }
    }
    overrides
}

fn option_true(value: &Value) -> bool {
    match value {
        Value::Array(values) => values.first().is_some_and(option_true),
        Value::Bool(value) => *value,
        Value::String(value) => value == "1" || value == "true",
        Value::Number(value) => value.as_i64() == Some(1),
        Value::Null | Value::Object(_) => false,
    }
}

fn first_number(value: &Value) -> Option<f64> {
    match value {
        Value::Array(values) => values.first().and_then(first_number),
        Value::String(value) if !value.ends_with('%') => value.parse().ok(),
        Value::Number(value) => value.as_f64(),
        _ => None,
    }
}

fn clamp_vector(
    fields: &Map<String, Value>,
    key: &str,
    minimum: f64,
    maximum: f64,
) -> Option<Value> {
    let Value::Array(values) = fields.get(key)? else {
        return None;
    };
    let mut changed = false;
    let values = values
        .iter()
        .map(|value| {
            let Some(number) = first_number(value) else {
                return value.clone();
            };
            let clamped = number.clamp(minimum, maximum);
            changed |= clamped != number;
            Value::String(if clamped.fract() == 0.0 {
                format!("{clamped:.0}")
            } else {
                clamped.to_string()
            })
        })
        .collect();
    changed.then_some(Value::Array(values))
}

pub(crate) fn vendors(root: &std::path::Path) -> Vec<String> {
    let Ok(entries) = std::fs::read_dir(root) else {
        return Vec::new();
    };
    let mut vendors = Vec::new();
    for entry in entries.flatten() {
        if let Some(name) = entry
            .path()
            .is_dir()
            .then(|| entry.file_name())
            .and_then(|name| name.to_str().map(ToOwned::to_owned))
        {
            vendors.push(name);
        }
    }
    vendors.sort();
    vendors
}

#[allow(dead_code)]
fn _type_assertions(map: &BTreeMap<String, Value>) {
    let _ = map.len();
    let _: Option<&Runner> = None;
}
