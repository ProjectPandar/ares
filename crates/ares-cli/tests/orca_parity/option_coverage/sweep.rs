use super::*;
use crate::runner::CaseInputs;
use sha2::{Digest, Sha256};

#[path = "sweep_tests.rs"]
mod tests;

#[test]
fn orca_parity_option_coverage() {
    if !crate::runner::reference_enabled("ARES_PARITY_OPTIONS") {
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
            plan.cases.retain(|case| selected_value(case, values));
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
    require_complete(&outcomes).unwrap_or_else(|error| panic!("{error}"));
}

fn selected_value(case: &domains::OptionCase, values: &std::collections::BTreeSet<String>) -> bool {
    case.value.as_ref().is_none_or(|v| {
        let label = v
            .as_str()
            .map(str::to_owned)
            .unwrap_or_else(|| v.to_string());
        values
            .iter()
            .any(|want| want.trim_matches('"') == label.trim_matches('"'))
    })
}

pub(super) fn require_complete(outcomes: &[OptionOutcome]) -> Result<(), String> {
    if outcomes.is_empty() {
        return Err("empty option coverage selection; no domains compared".into());
    }
    let incomplete = outcomes
        .iter()
        .filter(|outcome| outcome.status != "PASS")
        .count();
    if incomplete == 0 {
        Ok(())
    } else {
        Err(format!(
            "{incomplete} option domains incomplete or failed; see external option-coverage-summary.md"
        ))
    }
}

/// `(option, value)` pairs whose upstream oracle is nondeterministic by
/// construction — verified by two identical-manifest Orca runs producing
/// different G-code (`/tmp/parity-lockedzag-trav4/case-AKqSWX` vs
/// `case-lXtwTS`, 407-line oracle-vs-oracle diff).
pub(super) fn oracle_nondeterministic(key: &str, label: &str) -> bool {
    // Coherent-noise fuzzy modes sample point offsets with a thread-local
    // `std::mt19937` seeded by `random_device` (`FuzzySkin.cpp:24-30`):
    // two identical-manifest oracle runs diverge by ~13k lines
    // (`/tmp/orc1/plate_1.gcode` vs `/tmp/orc2/plate_1.gcode`). The sweep
    // baseline pins `fuzzy_skin_noise_type = ripple` (deterministic sine
    // offsets, no `random_value`), so only the coherent-noise labels and
    // the domains that force `noise_type = perlin` are excused.
    matches!(
        (key, label),
        ("fuzzy_skin_noise_type", "billow")
            | ("fuzzy_skin_noise_type", "classic")
            | ("fuzzy_skin_noise_type", "perlin")
            | ("fuzzy_skin_noise_type", "voronoi")
            | ("fuzzy_skin_noise_type", "ridgedmulti")
    ) || matches!(
        key,
        "fuzzy_skin_octaves" | "fuzzy_skin_persistence" | "fuzzy_skin_scale"
    )
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
    if plan.key == "filament_self_index" {
        return super::filament_self_index::execute_plan(runner, plan);
    }
    if plan.key == "master_extruder_id" {
        return super::master_extruder_id::execute_plan(runner, plan);
    }
    if plan.key == super::print_compatible_printers::KEY {
        // Replay embedded requests, never load this metadata key as a preset.
        // All seven witnesses and the retained baseline must pass before credit.
        super::print_compatible_printers::witnesses(runner);
        return OptionOutcome::executed(plan, plan.cases.len(), Vec::new(), 0, None);
    }
    if matches!(
        plan.key.as_str(),
        "printer_extruder_id" | "physical_extruder_map"
    ) {
        return super::printer_physical_identity::execute_plan(runner, plan);
    }
    #[cfg(target_os = "linux")]
    if plan.key == "post_process" {
        return super::post_process_matrix::execute_plan(runner, plan);
    }
    if plan.key == "print_extruder_id" {
        return super::print_extruder_identity::execute_plan(runner, plan);
    }
    if plan.key == "brim_ears" {
        return super::legacy_brim_ears::execute_plan(runner, plan);
    }
    if plan.key == "nozzle_flush_dataset" {
        return super::nozzle_dataset::execute_plan(runner, plan);
    }
    if plan.key == "filament_ramming_parameters" {
        return super::semm_ramming::execute_plan(runner, plan);
    }
    if plan.key == "other_layers_print_sequence_nums" {
        return super::tool_sequences::execute_plan(runner, plan);
    }
    let mut first_failure = None;
    let mut compared = 0;
    let mut excused = 0;
    let mut rejected = Vec::new();
    let io_error = |error| {
        runner::stages::StageError::new(
            runner::stages::Stage::Application,
            runner::stages::FailureKind::Io,
            error,
        )
    };
    let verify_application = |project: &[u8], value: &Value| {
        if plan.key == "filament_map_mode" {
            runner::filament_map::verify(project, value)
        } else if plan.key == super::adaptive_pa_bridges::KEY {
            super::adaptive_pa_bridges::verify(project, value)
        } else if matches!(
            plan.key.as_str(),
            "layer_height" | "initial_layer_print_height"
        ) {
            height_context::verify(project, &plan.key, value)
        } else if matches!(plan.key.as_str(), "bridge_flow" | "internal_bridge_flow") {
            let application = runner::application::verify_value(project, &plan.key, value)?;
            Ok(serde_json::json!({"application":application,
                "domain":super::bridge_flow_coverage::input::metadata(&plan.key, true)}))
        } else if matches!(
            plan.key.as_str(),
            "filament_flow_ratio" | "filament_diameter"
        ) {
            let application = runner::application::verify_value(project, &plan.key, value)?;
            Ok(serde_json::json!({"application":application,
                "domain":super::filament_flow_coverage::input::metadata()}))
        } else if plan.key == "timelapse_type" {
            let application = runner::application::verify_value(project, &plan.key, value)?;
            Ok(serde_json::json!({"application":application,
                "domain":super::timelapse_coverage::input::metadata()}))
        } else if plan.key == "wrapping_exclude_area" {
            let application = runner::application::verify_value(project, &plan.key, value)?;
            Ok(serde_json::json!({"application":application,
                "domain":super::wrapping_exclude_coverage::input::metadata()}))
        } else if super::tests::clearance::owns(&plan.key) {
            super::tests::clearance::verify_application(project, &plan.key, value)
        } else {
            runner::application::verify_value(project, &plan.key, value)
        }
    };
    for case in &plan.cases {
        let label_digest: String = Sha256::digest(case.label.as_bytes())
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect();
        let proof_name = format!("{}-{label_digest}-application.json", plan.key);
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
        let exported = if plan.key == "timelapse_type" {
            super::timelapse_coverage::input::export(
                runner,
                case.value.as_ref().unwrap(),
                true,
                &label,
            )
        } else if plan.key == "filament_map" {
            tool_context::mapping(runner, &label, case.value.as_ref().unwrap())
        } else if matches!(plan.key.as_str(), "bridge_flow" | "internal_bridge_flow") {
            super::bridge_flow_coverage::input::export(
                runner,
                &plan.key,
                case.value.as_ref().unwrap().as_str().unwrap(),
                true,
                &label,
            )
        } else if matches!(
            plan.key.as_str(),
            "filament_flow_ratio" | "filament_diameter"
        ) {
            super::filament_flow_coverage::input::export(
                runner,
                &plan.key,
                case.value.as_ref().unwrap()[0].as_str().unwrap(),
                &label,
            )
        } else if plan.key == "wrapping_exclude_area" {
            super::wrapping_exclude_coverage::input::export(
                runner,
                case.value.as_ref().unwrap(),
                true,
                &label,
            )
        } else {
            runner.export_case(
                &CaseInputs {
                    label: &label,
                    machine: &case_machine,
                    process: &case_process,
                    filaments: &case_filaments,
                },
                &overrides,
                &runner::repo_root().join(
                    if plan.key == super::adaptive_pa_bridges::KEY
                        || support_geometry_option(&plan.key)
                    {
                        // Support-family options are inert on a solid cube —
                        // the T-shape produces real support + interface layers.
                        "tests/parity/overhang.stl"
                    } else {
                        "tests/parity/cube10.stl"
                    },
                ),
            )
        };
        let built = exported
            .and_then(|exported| {
                if plan.key == "filament_map_mode" {
                    runner.bind_filament_map_mode(exported, case.value.as_ref().unwrap())
                } else {
                    Ok(exported)
                }
            })
            .and_then(|exported| {
                // Requested-vs-effective verification for EVERY injected
                // value: a case can only credit coverage when the exported
                // project actually carries the probed value (width domains
                // additionally type-check the percent/literal tag).
                if let Some(value) = &case.value {
                    let proof = verify_application(&exported.project, value)?;
                    let root = parity::artifacts::root_from_env().map_err(io_error)?;
                    let file = root.join(&proof_name);
                    parity::artifacts::overwrite(
                        &file,
                        &serde_json::to_vec_pretty(&proof).unwrap(),
                    )
                    .map_err(io_error)?;
                }
                if let Some(domain) = &plan.width_domain {
                    let proof = runner::application::verify(
                        &exported.project,
                        &plan.key,
                        case.value.as_ref().unwrap(),
                        domain.metadata["dependencies"].as_object().unwrap(),
                    )?;
                    let root = parity::artifacts::root_from_env().map_err(io_error)?;
                    let file = root.join(&proof_name);
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
                let outcome = if plan.key == "timelapse_type" {
                    super::timelapse_coverage::compare(
                        runner,
                        &exported,
                        case.value.as_ref().unwrap(),
                    )
                } else if plan.key == super::adaptive_pa_bridges::KEY {
                    super::adaptive_pa_bridges::compare(
                        runner,
                        &exported,
                        case.value.as_ref().unwrap(),
                    )
                } else {
                    parity::compare_exported_for(runner, &exported, Some(&plan.key))
                };
                // Only genuine byte divergence is excused — build/artifact
                // errors (ARES_ERROR, ORCA_ERROR, INPUT_REJECTED, ...) must
                // still surface as failures.
                if oracle_nondeterministic(&plan.key, &case.label) && outcome.status == "DIVERGENT"
                {
                    // Feature/FuzzySkin/FuzzySkin.cpp:26-31 random_value uses a
                    // thread-local entropy/thread-ID-seeded generator. These
                    // excused divergences remain incomplete coverage.
                    excused += 1;
                    eprintln!("[option] ORACLE_NONDETERMINISTIC {label}");
                    continue;
                }
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
    OptionOutcome::executed(plan, compared, rejected, excused, first_failure)
}
