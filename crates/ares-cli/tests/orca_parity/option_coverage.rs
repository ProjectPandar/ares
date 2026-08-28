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
        let mut first_failure = None;
        for case in &plan.cases {
            let mut case_machine = machine.clone();
            let mut case_process = process.clone();
            let mut case_filaments = filaments.clone();
            inject_case(
                plan,
                case,
                &mut case_machine,
                &mut case_process,
                &mut case_filaments,
            );
            let label = format!("option/{}/{}", plan.key, case.label);
            // The case value must win over the baseline smoke overrides;
            // drop the key from the override map so the injected preset
            // value survives (`option-coverage` requirement 3).
            let mut overrides = parity::smoke_overrides();
            overrides.remove(&plan.key);
            let built = runner.build_case(
                &CaseInputs {
                    label: &label,
                    machine: &case_machine,
                    process: &case_process,
                    filaments: &case_filaments,
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

fn inject_case(
    plan: &domains::OptionPlan,
    case: &domains::OptionCase,
    machine: &mut Map<String, Value>,
    process: &mut Map<String, Value>,
    filaments: &mut [Map<String, Value>],
) {
    let target = match plan.raw_scope.as_str() {
        "printer" => machine,
        "process" => process,
        "filament" => &mut filaments[0],
        "residual" if residual_is_machine(&plan.key) => machine,
        "residual" => process,
        scope => panic!("unknown option scope {scope}"),
    };
    target.insert(plan.key.clone(), case.value.clone());
}

fn residual_is_machine(key: &str) -> bool {
    matches!(
        key,
        "deretraction_speed"
            | "extruder_offset"
            | "max_layer_height"
            | "min_layer_height"
            | "nozzle_diameter"
            | "nozzle_volume_type"
            | "retract_before_wipe"
            | "retract_length_toolchange"
            | "retract_lift_above"
            | "retract_lift_below"
            | "retract_restart_extra"
            | "retract_restart_extra_toolchange"
            | "retraction_length"
            | "retraction_minimum_travel"
            | "retraction_speed"
            | "wipe"
            | "wipe_distance"
            | "z_hop"
    )
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
