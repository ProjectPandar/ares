use crate::{
    self as parity,
    presets::VendorProfiles,
    runner::{self, OrcaRunner},
};

use super::{cube_model, profiles_root};

/// Full vendor sweep; writes `tests/parity/printer-smoke-summary.md` and
/// fails when any printer diverges, so each fix moves the summary to green.
/// Gated behind `ARES_PARITY_SWEEP=1` because slicing every vendor preset
/// through both slicers takes hours.
pub(super) fn run_printer_sweep() {
    if std::env::var("ARES_PARITY_SWEEP").as_deref() != Ok("1") {
        eprintln!("skipping: set ARES_PARITY_SWEEP=1 to run the full printer sweep");
        return;
    }
    let runner = OrcaRunner::from_env()
        .unwrap_or_else(|e| panic!("{e}"))
        .expect("explicit sweep requires an Orca executable");
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
    // `ORCA_ERROR` and `VENDOR_INCOMPLETE` are documented exemptions —
    // the upstream binary cannot produce a reference (or the vendor
    // tree lacks the referenced preset), so Ares correctness is
    // unverifiable there rather than divergent. Genuine parity failures
    // (`DIVERGENT`, `ARES_ERROR`, `INPUT_REJECTED`) still fail the
    // sweep; the exempt rows stay visible in the summary.
    let failures = outcomes
        .iter()
        .filter(|outcome| {
            matches!(
                outcome.status,
                "DIVERGENT" | "ARES_ERROR" | "INPUT_REJECTED"
            )
        })
        .count();
    assert!(
        failures == 0,
        "{failures}/{} printers diverge; see tests/parity/printer-smoke-summary.md",
        outcomes.len()
    );
}

fn write_summary(outcomes: &[parity::ParityOutcome]) {
    let passed = outcomes
        .iter()
        .filter(|outcome| outcome.status == "PASS")
        .count();
    let mut summary = format!(
        "# OrcaSlicer printer smoke summary\n\n{} of {} printers pass the strict ordered-byte comparison (generator identity/timestamp lines normalized; classic wall generator baseline; cube model).\n\nStatuses: `PASS` (normalized byte equality), `DIVERGENT` (first byte difference), `ORCA_ERROR` (the upstream OrcaSlicer 2.4.2 reference binary itself failed, so no reference stream exists), `VENDOR_INCOMPLETE` (the vendor profile tree does not ship the machine's referenced default process preset), `ARES_ERROR` (Ares failed to load or slice the case).\n\n| status | printer | first divergence |\n|---|---|---|\n",
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
