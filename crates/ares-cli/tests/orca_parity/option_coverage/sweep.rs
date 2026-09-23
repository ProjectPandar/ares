use super::*;
use crate::runner::CaseInputs;

#[test]
fn orca_parity_option_coverage() {
    if std::env::var("ARES_PARITY_OPTIONS").as_deref() != Ok("1") {
        eprintln!("skipping: set ARES_PARITY_OPTIONS=1 to run option coverage");
        return;
    }
    let runner = OrcaRunner::from_env()
        .unwrap_or_else(|e| panic!("{e}"))
        .expect("explicit option coverage requires Orca and external artifacts");
    let profiles = VendorProfiles::load(&profiles_root(), "Creality").unwrap();
    let selection =
        parity::select_printer(&profiles, "Creality", "Creality Ender-3 0.4 nozzle").unwrap();
    let machine = profiles.machine(&selection.printer).unwrap();
    let mut process = profiles.process(&selection.process).unwrap();
    parity::normalize_process_defaults(&machine, &mut process);
    let mut filaments = selection
        .filaments
        .iter()
        .map(|name| profiles.filament(name).unwrap())
        .collect::<Vec<_>>();
    parity::normalize_filament_defaults(&mut filaments);
    // `ARES_OPTION_KEYS=a,b,c` narrows the sweep to named options for
    // targeted re-verification; unset sweeps the whole inventory.
    let only: Option<std::collections::BTreeSet<String>> = std::env::var("ARES_OPTION_KEYS")
        .ok()
        .map(|list| list.split(',').map(str::trim).map(str::to_owned).collect());
    // `ARES_OPTION_VALUES=a,b,c` further narrows each selected plan's cases to
    // the named option values (e.g. `rectilinear,monotonic`); unset runs every
    // case in the plan's domain. `ARES_PARITY_PATTERN` is accepted as an alias.
    let only_values: Option<std::collections::BTreeSet<String>> =
        std::env::var("ARES_OPTION_VALUES")
            .ok()
            .or_else(|| std::env::var("ARES_PARITY_PATTERN").ok())
            .map(|list| list.split(',').map(str::trim).map(str::to_owned).collect());
    let plans = domains::load(&runner::repo_root());
    let mut plans: Vec<_> = plans
        .into_iter()
        .filter(|plan| only.as_ref().is_none_or(|set| set.contains(&plan.key)))
        .collect();
    if let Some(set) = &only {
        let missing: Vec<_> = set
            .iter()
            .filter(|key| !plans.iter().any(|plan| &plan.key == *key))
            .collect();
        assert!(
            missing.is_empty(),
            "ARES_OPTION_KEYS selected unknown option keys {missing:?}"
        );
    }
    if let Some(values) = &only_values {
        for plan in &mut plans {
            let before = plan.cases.len();
            plan.cases.retain(|case| {
                case.value.as_ref().is_none_or(|v| {
                    let label = v
                        .as_str()
                        .map(str::to_owned)
                        .unwrap_or_else(|| v.to_string());
                    values
                        .iter()
                        .any(|want| want.trim_matches('"') == label.trim_matches('"'))
                })
            });
            // A value-filtered run no longer executes the plan's full legal
            // domain — clear `provably_legal` so the summary can't report a
            // partial (single-value) run as a complete domain.
            if plan.cases.len() != before {
                plan.provably_legal = false;
            }
        }
    }
    let outcomes = plans
        .iter()
        .map(|plan| execute_plan(&runner, plan, &machine, &process, &filaments))
        .collect::<Vec<_>>();
    write_summary(&outcomes);
    let incomplete = outcomes
        .iter()
        .filter(|outcome| outcome.status != "PASS")
        .count();
    assert_eq!(
        incomplete, 0,
        "{incomplete} option domains incomplete or failed; see external option-coverage-summary.md"
    );
}

fn execute_plan(
    runner: &OrcaRunner,
    plan: &domains::OptionPlan,
    machine: &Map<String, Value>,
    process: &Map<String, Value>,
    filaments: &[Map<String, Value>],
) -> OptionOutcome {
    if plan.cases.is_empty() {
        return OptionOutcome::omitted(plan);
    }
    let mut first_failure = None;
    let mut compared = 0;
    let mut rejected = Vec::new();
    let io_error = |error| {
        runner::stages::StageError::new(
            runner::stages::Stage::Application,
            runner::stages::FailureKind::Io,
            error,
        )
    };
    for case in &plan.cases {
        let mut case_machine = machine.clone();
        let mut case_process = process.clone();
        let mut case_filaments = filaments.to_vec();
        inject_case(
            plan,
            case,
            &mut case_machine,
            &mut case_process,
            &mut case_filaments,
        );
        let label = format!("option/{}/{}", plan.key, case.label);
        let mut overrides = parity::smoke_overrides();
        overrides.remove(&plan.key);
        let built = runner
            .export_case(
                &CaseInputs {
                    label: &label,
                    machine: &case_machine,
                    process: &case_process,
                    filaments: &case_filaments,
                },
                &overrides,
                &runner::repo_root().join("tests/parity/cube10.stl"),
            )
            .and_then(|exported| {
                if let Some(domain) = &plan.width_domain {
                    let proof = runner::application::verify(
                        &exported.project,
                        &plan.key,
                        case.value.as_ref().unwrap(),
                        domain.metadata["dependencies"].as_object().unwrap(),
                    )?;
                    let root = parity::artifacts::root_from_env().map_err(io_error)?;
                    let file = root.join(format!("{}-{}-application.json", plan.key, case.label));
                    parity::artifacts::overwrite(
                        &file,
                        &serde_json::to_vec_pretty(&proof).unwrap(),
                    )
                    .map_err(io_error)?;
                }
                Ok(exported)
            });
        match built {
            Ok(exported) => {
                let outcome = parity::compare_exported_for(runner, &exported, Some(&plan.key));
                if outcome.status == "INPUT_REJECTED" {
                    rejected.push(format!("{}: {}", case.label, outcome.detail));
                    eprintln!("[option] INPUT_REJECTED {label}");
                    continue;
                }
                compared += usize::from(strict_comparator_ran(outcome.status));
                eprintln!("[option] {} {label}", outcome.status);
                if outcome.status != "PASS" && first_failure.is_none() {
                    first_failure = Some(format!("{}: {}", case.label, outcome.detail));
                }
            }
            Err(error) if error.input_rejection(&plan.key) => {
                rejected.push(format!("{}: {error}", case.label));
                eprintln!("[option] INPUT_REJECTED {label}");
            }
            Err(error) => {
                eprintln!("[option] STAGE_FAILURE {label}: {error}");
                first_failure.get_or_insert_with(|| format!("{}: {error}", case.label));
            }
        }
    }
    OptionOutcome::executed(plan, compared, rejected, first_failure)
}
