use crate::{
    self as parity,
    presets::VendorProfiles,
    runner::{self, OrcaRunner},
};

use super::{cube_model, profiles_root};

#[path = "sweep/tests.rs"]
mod tests;

#[path = "sweep/prime_tower_export.rs"]
mod prime_tower_export;

/// Full vendor sweep; writes `tests/parity/printer-smoke-summary.md` and
/// fails unless every printer produces a matching complete output.
/// Gated behind `ARES_PARITY_SWEEP=1` because slicing every vendor preset
/// through both slicers takes hours.
pub(super) fn run_printer_sweep() {
    if !crate::runner::reference_enabled("ARES_PARITY_SWEEP") {
        eprintln!("skipping: set ARES_PARITY_SWEEP=1 to run the full printer sweep");
        return;
    }
    let runner = OrcaRunner::from_env()
        .unwrap_or_else(|e| panic!("{e}"))
        .expect("explicit sweep requires an Orca executable");
    // Preflight: the oracle wrapper must actually execute — a missing
    // `ORCA_APPDIR` exits 127 before producing any reference, and the
    // sweep would otherwise record only `ORCA_ERROR` outcomes and pass
    // without a single comparison.
    let preflight = std::process::Command::new(runner.bin())
        .arg("--help")
        .output()
        .expect("spawn Orca oracle preflight");
    assert!(
        preflight.status.success(),
        "Orca oracle preflight failed (status {:?}) — set ORCA_APPDIR to the extracted official AppImage: {}",
        preflight.status.code(),
        String::from_utf8_lossy(&preflight.stderr).trim()
    );
    let root = profiles_root();
    let model = cube_model();
    let vendors = parity::vendors(&root);
    // A readable profile root must enumerate at least one vendor; an empty
    // list would silently turn the sweep into a vacuous 0/0 pass.
    assert!(
        !vendors.is_empty(),
        "printer sweep found no vendors under {root:?} — profiles root unreadable or empty"
    );
    let mut outcomes = Vec::new();
    for vendor in vendors {
        let profiles = match VendorProfiles::load(&root, &vendor) {
            Ok(profiles) => profiles,
            Err(error) => {
                // Count an unloadable vendor as an outcome, not a silent skip.
                let outcome =
                    parity::ares_error(&format!("{vendor}/<vendor-load>"), error.to_string());
                eprintln!(
                    "[{}/sweep] {} {}",
                    outcomes.len() + 1,
                    outcome.status,
                    outcome.label
                );
                outcomes.push(outcome);
                continue;
            }
        };
        for printer in profiles.instantiated_machine_names() {
            let selection = match parity::select_printer(&profiles, &vendor, &printer) {
                Ok(selection) => selection,
                Err(error) => {
                    let outcome = parity::ares_error(&format!("{vendor}/{printer}"), error);
                    eprintln!(
                        "[{}/sweep] {} {}",
                        outcomes.len() + 1,
                        outcome.status,
                        outcome.label
                    );
                    outcomes.push(outcome);
                    continue;
                }
            };
            let outcome =
                match parity::export_selection_case(&runner, &profiles, &selection, &model) {
                    Ok(exported) => parity::compare_exported(&runner, &exported),
                    Err(error) => parity::ares_error(&format!("{vendor}/{printer}"), error),
                };
            eprintln!(
                "[{}/sweep] {} {}",
                outcomes.len() + 1,
                outcome.status,
                outcome.label
            );
            outcomes.push(outcome);
        }
    }
    write_summary(&outcomes);
    require_complete(&outcomes);
}

fn require_complete(outcomes: &[parity::ParityOutcome]) {
    assert!(
        !outcomes.is_empty(),
        "empty printer sweep; no printers compared"
    );
    let incomplete = outcomes
        .iter()
        .filter(|outcome| outcome.status != "PASS")
        .count();
    assert!(
        incomplete == 0,
        "{incomplete}/{} printers incomplete or failed; see tests/parity/printer-smoke-summary.md",
        outcomes.len()
    );
}

fn write_summary(outcomes: &[parity::ParityOutcome]) {
    let passed = outcomes
        .iter()
        .filter(|outcome| outcome.status == "PASS")
        .count();
    let mut summary = format!(
        "# OrcaSlicer printer smoke summary\n\n{} of {} printers pass the strict ordered-byte comparison (generator identity/timestamp lines normalized; cube model).\n\nThis matrix pins a MODIFIED baseline (`smoke_overrides`: `wall_generator=classic`, `detect_thin_wall=0`, bed-exclude and post-process cleared, one filament) — it verifies parity of the ported classic-wall slice across every printer's machine/process parameterization, not untouched default profiles. Arachne, thin-wall detection, multi-extruder, and vendor post-process paths are upstream boundaries pending port; an untouched-profile acceptance matrix is deferred to those milestones.\n\nStatuses: `PASS` (normalized byte equality), `DIVERGENT` (first byte difference), `ORCA_ERROR` (the upstream OrcaSlicer 2.4.2 reference binary itself failed, so no reference stream exists), `VENDOR_INCOMPLETE` (the vendor profile tree does not ship the machine's referenced default process preset), `ARES_ERROR` (Ares failed to load or slice the case).\n\n| status | printer | first divergence |\n|---|---|---|\n",
        passed,
        outcomes.len()
    );
    for outcome in outcomes {
        summary.push_str(&format!(
            "| {} | {} | {} |\n",
            outcome.status,
            outcome.label,
            outcome.detail.replace('\n', " ")
        ));
    }
    let path = runner::repo_root().join("tests/parity/printer-smoke-summary.md");
    std::fs::write(&path, summary).expect("write printer smoke summary");
}
