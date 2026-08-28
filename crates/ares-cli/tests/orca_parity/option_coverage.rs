//! Option-domain parity sweep against the Ender-3 smoke fixture.

#[path = "option_coverage/domains.rs"]
mod domains;
#[cfg(test)]
#[path = "option_coverage/tests.rs"]
mod tests;

use std::path::PathBuf;

use crate::{
    self as parity,
    presets::VendorProfiles,
    runner::{self, CaseInputs, OrcaRunner},
};
use serde_json::{Map, Value};

fn profiles_root() -> PathBuf {
    runner::repo_root().join("OrcaSlicer/resources/profiles")
}

#[test]
fn orca_parity_option_coverage() {
    if std::env::var("ARES_PARITY_OPTIONS").as_deref() != Ok("1") {
        eprintln!("skipping: set ARES_PARITY_OPTIONS=1 to run option coverage");
        return;
    }
    let Some(runner) = OrcaRunner::from_env() else {
        eprintln!("skipping: no OrcaSlicer CLI available");
        return;
    };
    let profiles = VendorProfiles::load(&profiles_root(), "Creality").unwrap();
    let selection =
        parity::select_printer(&profiles, "Creality", "Creality Ender-3 0.4 nozzle").unwrap();
    let machine = profiles.machine(&selection.printer).unwrap();
    let process = profiles.process(&selection.process).unwrap();
    let filaments = selection
        .filaments
        .iter()
        .map(|name| profiles.filament(name).unwrap())
        .collect::<Vec<_>>();
    let plans = domains::load(&runner::repo_root());
    let mut outcomes = Vec::new();
    for plan in &plans {
        if plan.cases.is_empty() {
            outcomes.push(OptionOutcome::omitted(plan));
            continue;
        }
        let owned = machine.contains_key(&plan.key)
            || process.contains_key(&plan.key)
            || filaments
                .iter()
                .any(|filament| filament.contains_key(&plan.key));
        if !owned {
            outcomes.push(OptionOutcome::missing(plan));
            continue;
        }
        let mut first_failure = None;
        for case in &plan.cases {
            let mut overrides = parity::smoke_overrides();
            overrides.insert(plan.key.clone(), case.value.clone());
            let label = format!("option/{}/{}", plan.key, case.label);
            let built = runner.build_case(
                &CaseInputs {
                    label: &label,
                    machine: &machine,
                    process: &process,
                    filaments: &filaments,
                },
                &overrides,
                &runner::repo_root().join("tests/parity/cube10.stl"),
            );
            let outcome = match built {
                Ok(case) => parity::compare_case(&case),
                Err(error) => parity::ares_error(&label, error),
            };
            eprintln!("[option] {} {}", outcome.status, label);
            if outcome.status != "PASS" && first_failure.is_none() {
                first_failure = Some(format!("{}: {}", case.label, outcome.detail));
            }
        }
        outcomes.push(OptionOutcome::executed(plan, first_failure));
    }
    write_summary(&outcomes);
    let failures = outcomes
        .iter()
        .filter(|outcome| outcome.status == "FAIL" || outcome.status == "MISSING")
        .count();
    assert_eq!(
        failures, 0,
        "{failures} option domains fail; see tests/parity/option-coverage-summary.md"
    );
}

struct OptionOutcome {
    key: String,
    option_type: String,
    source: String,
    cases: usize,
    status: &'static str,
    detail: String,
}

impl OptionOutcome {
    fn executed(plan: &domains::OptionPlan, failure: Option<String>) -> Self {
        let (status, detail) = failure.map_or(("PASS", String::new()), |detail| ("FAIL", detail));
        Self::new(plan, plan.cases.len(), status, detail)
    }

    fn omitted(plan: &domains::OptionPlan) -> Self {
        Self::new(
            plan,
            0,
            "UNBOUNDED",
            plan.omission.unwrap_or_default().to_owned(),
        )
    }

    fn missing(plan: &domains::OptionPlan) -> Self {
        Self::new(
            plan,
            plan.cases.len(),
            "MISSING",
            "option absent from Ender-3 baseline presets".to_owned(),
        )
    }

    fn new(plan: &domains::OptionPlan, cases: usize, status: &'static str, detail: String) -> Self {
        Self {
            key: plan.key.clone(),
            option_type: plan.option_type.clone(),
            source: plan.source.clone(),
            cases,
            status,
            detail,
        }
    }
}

fn write_summary(outcomes: &[OptionOutcome]) {
    let pass = outcomes
        .iter()
        .filter(|outcome| outcome.status == "PASS")
        .count();
    let executed_cases = outcomes
        .iter()
        .filter(|outcome| matches!(outcome.status, "PASS" | "FAIL"))
        .map(|outcome| outcome.cases)
        .sum::<usize>();
    let mut output = format!(
        "# OrcaSlicer option coverage summary\n\n{pass} of {} executable option domains pass ({executed_cases} generated cases).\n\n| status | option | type | cases | upstream | first result |\n|---|---|---|---:|---|---|\n",
        outcomes
            .iter()
            .filter(|outcome| matches!(outcome.status, "PASS" | "FAIL"))
            .count()
    );
    for outcome in outcomes {
        output.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} |\n",
            outcome.status,
            outcome.key,
            outcome.option_type,
            outcome.cases,
            outcome.source,
            outcome.detail.replace(['\n', '|'], " "),
        ));
    }
    std::fs::write(
        runner::repo_root().join("tests/parity/option-coverage-summary.md"),
        output,
    )
    .unwrap();
}

#[allow(dead_code)]
fn _type_assertion(_: &Map<String, Value>) {}
