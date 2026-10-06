//! OrcaSlicer parity suite: slices vendor printer profiles with both the
//! OrcaSlicer 2.4.2 CLI and Ares using the shared generator-only strict byte
//! comparator. Equality of supplied streams does not establish inventory coverage.
//!
//! Environment-gated: runs only when `ARES_ORCA_BIN` (or the repository
//! wrapper `scripts/orca-parity.sh`) names a working OrcaSlicer CLI.

#[path = "orca_parity/adaptive_pressure_advance.rs"]
mod adaptive_pressure_advance;
#[path = "orca_parity/artifacts.rs"]
mod artifacts;
#[path = "orca_parity/brim_bed_region.rs"]
mod brim_bed_region;
#[path = "orca_parity/brim_bed_region_controls.rs"]
mod brim_bed_region_controls;
#[path = "orca_parity/bridge_over_infill_filament.rs"]
mod bridge_over_infill_filament;
#[path = "orca_parity/curled_perimeter_slowdown.rs"]
mod curled_perimeter_slowdown;
#[path = "orca_parity/empty_selected_plate.rs"]
mod empty_selected_plate;
#[path = "orca_parity/empty_selected_plate_controls.rs"]
mod empty_selected_plate_controls;
#[path = "orca_parity/filament_usage_stats.rs"]
mod filament_usage_stats;
#[path = "orca_parity/filament_volume_oracle.rs"]
mod filament_volume_oracle;
#[path = "orca_parity/flow_errors.rs"]
mod flow_errors;
#[path = "orca_parity/honeycomb_multiline.rs"]
mod honeycomb_multiline;
#[path = "orca_parity/option_coverage.rs"]
mod option_coverage;
#[path = "orca_parity/presets.rs"]
mod presets;
#[path = "orca_parity/print_validation.rs"]
mod print_validation;
#[path = "orca_parity/replay.rs"]
mod replay;
#[path = "orca_parity/replay_tests.rs"]
mod replay_tests;
#[path = "orca_parity/runner.rs"]
mod runner;
#[path = "orca_parity/smoke.rs"]
mod smoke;
#[path = "orca_parity/smoke_overrides.rs"]
mod smoke_overrides;
#[path = "orca_parity/smooth_type2_tower.rs"]
mod smooth_type2_tower;
#[path = "orca_parity/stock_presets.rs"]
mod stock_presets;
#[path = "orca_parity/sync_time.rs"]
mod sync_time;
#[path = "orca_parity/vg1.rs"]
mod vg1;

#[path = "ksr_fdmtest_v4/golden.rs"]
mod golden;
// Diagnostic-only semantic behavior tests; never a whole-output acceptance gate.
#[path = "ksr_fdmtest_v4/semantic.rs"]
mod semantic;

use std::collections::BTreeMap;

use presets::VendorProfiles;
use runner::{CaseInputs, OrcaRunner};
use serde_json::Value;
pub(crate) use smoke_overrides::{
    normalize_filament_defaults, normalize_process_defaults, smoke_case_overrides, smoke_overrides,
};

use crate::runner::{OrcaRunner as Runner, ParityCase};

pub(crate) struct ParityOutcome {
    pub(crate) label: String,
    pub(crate) status: &'static str,
    pub(crate) detail: String,
    pub(crate) artifacts: Option<std::path::PathBuf>,
}

pub(crate) fn compare_case(case: &ParityCase) -> ParityOutcome {
    artifacts::root_from_env()
        .and_then(|root| {
            artifacts::compare(
                &root,
                &case.label,
                Ok(&case.project),
                Ok(&case.reference),
                serde_json::json!({"kind": "runner_cache", "producer_identity": "unknown"}),
            )
        })
        .unwrap_or_else(|error| artifact_error(&case.label, error))
}

pub(crate) fn artifact_error(label: &str, error: String) -> ParityOutcome {
    ParityOutcome {
        label: label.to_owned(),
        status: "ARTIFACT_ERROR",
        detail: error,
        artifacts: None,
    }
}

pub(crate) fn pass(label: &str) -> ParityOutcome {
    ParityOutcome {
        label: label.to_owned(),
        status: "PASS",
        detail: String::new(),
        artifacts: None,
    }
}

pub(crate) fn divergence(label: &str, difference: String) -> ParityOutcome {
    ParityOutcome {
        label: label.to_owned(),
        status: "DIVERGENT",
        detail: difference,
        artifacts: None,
    }
}

pub(crate) fn ares_error(label: &str, error: String) -> ParityOutcome {
    // The upstream OrcaSlicer 2.4.2 binary itself failed (crash or nonzero
    // exit) while producing the reference stream; Ares was never invoked.
    if error.starts_with("Export/Process: unsuccessful Orca process")
        || error.starts_with("Slice/Process: unsuccessful Orca process")
    {
        return ParityOutcome {
            label: label.to_owned(),
            status: "ORCA_ERROR",
            detail: error,
            artifacts: None,
        };
    }
    // The vendor tree itself lacks the machine's referenced default preset
    // (and no compatible preset exists), so no reference producer input can
    // be assembled for this printer.
    if error.contains("default preset") && error.contains("not found")
        || error.contains("no compatible preset")
    {
        return ParityOutcome {
            label: label.to_owned(),
            status: "VENDOR_INCOMPLETE",
            detail: error,
            artifacts: None,
        };
    }
    ParityOutcome {
        label: label.to_owned(),
        status: "ARES_ERROR",
        detail: error,
        artifacts: None,
    }
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
    let exported = export_selection_case(runner, profiles, selection, model)?;
    runner
        .slice_case(&exported)
        .map_err(|error| error.to_string())
}

pub(crate) fn export_selection_case(
    runner: &OrcaRunner,
    profiles: &VendorProfiles,
    selection: &PrinterSelection,
    model: &std::path::Path,
) -> Result<runner::ExportedCase, String> {
    let machine = profiles.machine(&selection.printer)?;
    let mut process = profiles.process(&selection.process)?;
    normalize_process_defaults(&machine, &mut process);
    let mut filaments = selection
        .filaments
        .iter()
        .map(|name| profiles.filament(name))
        .collect::<Result<Vec<_>, _>>()?;
    normalize_filament_defaults(&mut filaments);
    let label = format!("{}/{}", selection.vendor, selection.printer);
    let overrides = smoke_case_overrides(&machine, &process);
    runner
        .export_case(
            &CaseInputs {
                label: &label,
                machine: &machine,
                process: &process,
                filaments: &filaments,
            },
            &overrides,
            model,
        )
        .map_err(|error| error.to_string())
}

/// Number of OrcaSlicer reference slices a diverging case may consume before
/// being declared DIVERGENT (`ARES_ORCA_RUNS`, default 3: the 2.4.2 oracle
/// can change tiny concentric survivors and M73 placement between runs).
pub(crate) fn oracle_run_budget() -> usize {
    std::env::var("ARES_ORCA_RUNS")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .filter(|runs| *runs >= 1)
        .unwrap_or(3)
}

/// Compares an exported case against the OrcaSlicer reference, re-slicing
/// the reference up to the configured run budget on divergence: the 2.4.2
/// oracle is itself non-deterministic across runs (observed: tiny concentric
/// loop survivors and M73 progress placement flip between runs of the same
/// binary), so parity means byte-equality with at least one genuine oracle
/// output, not with one arbitrarily chosen run. Every PASS requires the strict
/// ordered-byte comparator; an observed match does not prove oracle stability.
pub(crate) fn compare_exported(
    runner: &OrcaRunner,
    exported: &runner::ExportedCase,
) -> ParityOutcome {
    compare_exported_for(runner, exported, None)
}

/// `compare_exported` with an optional option key: when the oracle itself
/// rejects the slice with a diagnostic attributable to that key
/// (`stages::attributable`), the case is a legitimate input rejection, not
/// a divergence or an oracle-process error. The option sweep passes its key
/// so schema-legal-but-infeasible values (absolute-E `use_relative_e_`
/// `distances`, below-model `printable_height`) classify as rejections.
pub(crate) fn compare_exported_for(
    runner: &OrcaRunner,
    exported: &runner::ExportedCase,
    option_key: Option<&str>,
) -> ParityOutcome {
    let budget = oracle_run_budget();
    let mut attempt = 1;
    let mut outcome = match runner.slice_case(exported) {
        Ok(case) => compare_case(&case),
        Err(error) => {
            if let Some(outcome) = print_validation::rejections::compare(exported, &error) {
                return outcome;
            }
            if option_key.is_some_and(|key| error.input_rejection(key)) {
                return ParityOutcome {
                    label: exported.label.clone(),
                    status: "INPUT_REJECTED",
                    detail: error.to_string(),
                    artifacts: None,
                };
            }
            return ares_error(&exported.label, error.to_string());
        }
    };
    while outcome.status == "DIVERGENT" && attempt < budget {
        attempt += 1;
        match runner.rerun_reference(exported) {
            Ok(case) => {
                outcome = compare_case(&case);
            }
            // A failed retry must not erase an already-established
            // divergence — keep the mismatch and attach the retry
            // failure as evidence instead.
            Err(error) => {
                outcome.detail =
                    format!("{} (retry {attempt} oracle error: {error})", outcome.detail);
                break;
            }
        }
    }
    if outcome.status == "DIVERGENT" && attempt > 1 {
        outcome.detail = format!(
            "(no match across {attempt} completed oracle runs of {budget} budgeted) {}",
            outcome.detail
        );
    }
    outcome
}

pub(crate) fn vendors(root: &std::path::Path) -> Vec<String> {
    let Ok(entries) = std::fs::read_dir(root) else {
        return Vec::new();
    };
    let mut vendors = Vec::new();
    for entry in entries.flatten() {
        // Only directories that carry a `machine/` inventory contribute
        // printers — filament-only collections like `OrcaFilamentLibrary`
        // have no machine presets and previously produced a fake
        // ARES_ERROR row in the printer denominator.
        if let Some(name) = entry
            .path()
            .join("machine")
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
