use crate::{ParityOutcome, presets::VendorProfiles, runner::OrcaRunner};
use serde_json::json;

#[path = "discovery.rs"]
mod discovery;
#[path = "sweep_tests.rs"]
mod tests;

fn run() {
    if !crate::runner::reference_enabled("ARES_PARITY_STOCK_SWEEP") {
        return;
    }
    let root = crate::artifacts::root_from_env().unwrap();
    let runner = OrcaRunner::from_env().unwrap().unwrap();
    let profiles_root = crate::runner::repo_root().join("OrcaSlicer/resources/profiles");
    let model = crate::runner::repo_root().join("tests/parity/cube10.stl");
    let mut outcomes = Vec::new();
    let vendors = match discovery::vendors(&profiles_root) {
        Ok(vendors) => vendors,
        Err(error) => {
            outcomes.push(crate::ares_error("stock/<vendor-discovery>", error));
            finish(&root, &outcomes).unwrap_or_else(|error| panic!("{error}"));
            return;
        }
    };
    for vendor in vendors {
        let profiles = match VendorProfiles::load(&profiles_root, &vendor) {
            Ok(profiles) => profiles,
            Err(error) => {
                outcomes.push(crate::ares_error(
                    &format!("stock/{vendor}/<vendor-load>"),
                    error,
                ));
                continue;
            }
        };
        for printer in profiles.instantiated_machine_names() {
            let label = format!("stock/{vendor}/{printer}");
            let exported = crate::select_printer(&profiles, &vendor, &printer)
                .and_then(|selection| super::export(&runner, &profiles, &selection, &model));
            let outcome = match exported {
                Ok(exported) => crate::compare_exported(&runner, &exported),
                Err(error) => crate::ares_error(&label, error),
            };
            eprintln!(
                "[stock {}/sweep] {} {label}",
                outcomes.len() + 1,
                outcome.status
            );
            outcomes.push(outcome);
        }
    }
    finish(&root, &outcomes).unwrap_or_else(|error| panic!("{error}"));
}

fn finish(root: &std::path::Path, outcomes: &[ParityOutcome]) -> Result<(), String> {
    let rows = outcomes
        .iter()
        .map(|outcome| {
            json!({
                "printer": outcome.label,
                "status": outcome.status,
                "detail": outcome.detail,
                "artifacts": outcome.artifacts,
            })
        })
        .collect::<Vec<_>>();
    let passed = outcomes.iter().filter(|row| row.status == "PASS").count();
    let report = json!({
        "baseline": "stock presets; no smoke rewrites",
        "complete": !outcomes.is_empty() && passed == outcomes.len(),
        "passed_rows": passed,
        "outcome_rows": outcomes.len(),
        "rows": rows,
    });
    crate::artifacts::overwrite(
        &root.join("stock-printer-summary.json"),
        &serde_json::to_vec_pretty(&report).unwrap(),
    )?;
    if outcomes.is_empty() {
        Err("empty stock printer inventory; no printers compared".into())
    } else if passed != outcomes.len() {
        Err(format!(
            "{}/{} stock outcome rows incomplete; see {}/stock-printer-summary.json",
            outcomes.len() - passed,
            outcomes.len(),
            root.display()
        ))
    } else {
        Ok(())
    }
}
