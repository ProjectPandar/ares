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
    let process = pick_preset(
        base_process.as_deref(),
        &variant,
        |name| profiles.process_exists(name),
        &format!("{vendor}/{printer} process"),
    )?;

    let filaments: Vec<String> = machine
        .get("default_filament_profile")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .filter(|name| !name.is_empty() && profiles.filament_exists(name))
        .map(ToOwned::to_owned)
        .collect();
    if filaments.is_empty() {
        return Err(format!(
            "{vendor}/{printer}: no usable default_filament_profile"
        ));
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
    runner.build_case(
        &CaseInputs {
            label: &label,
            machine: &machine,
            process: &process,
            filaments: &filaments,
        },
        &smoke_overrides(),
        model,
    )
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
