use std::collections::BTreeMap;

use super::*;
use crate::runner::CaseInputs;

#[test]
fn orca_parity_generic_tree_branch_angle_changes_support_moves_and_preserves_complete_output() {
    let Some(moves) = compare_domain("tree_support_branch_angle") else {
        return;
    };
    assert!(
        moves["min"] != moves["max"],
        "generic branch-angle boundaries must affect native support movements"
    );
    assert!(moves["min"] != moves["seeded"]);
}

#[test]
fn orca_parity_tree_style_values_change_support_moves_and_preserve_complete_output() {
    let Some(moves) = compare_domain("support_style") else {
        return;
    };
    for style in ["tree_slim", "tree_strong", "tree_hybrid", "organic"] {
        assert!(
            moves[style] != moves["grid"],
            "{style} must not silently exercise Normal Grid support movements"
        );
    }
}

#[test]
fn orca_parity_classic_tree_dependencies_activate_support_moves_and_preserve_complete_output() {
    for (key, left, right) in [
        ("tree_support_branch_distance", "min", "max"),
        ("tree_support_branch_diameter", "min", "max"),
        ("tree_support_auto_brim", "false", "true"),
        ("tree_support_brim_width", "min", "seeded"),
    ] {
        let Some(moves) = compare_domain(key) else {
            return;
        };
        assert!(
            moves[left] != moves[right],
            "{key} must affect native support moves"
        );
    }
}

#[test]
fn orca_parity_organic_angle_and_normalized_grid_tip_preserve_complete_output() {
    for key in ["tree_support_angle_slow", "tree_support_tip_diameter"] {
        let Some(moves) = compare_domain(key) else {
            return;
        };
        assert!(
            moves["min"] != moves["max"],
            "{key} must affect native support moves"
        );
    }
}

#[test]
fn orca_parity_explicit_organic_tip_legal_boundaries_and_native_rejection_are_distinct() {
    let Some(runner) = reference_runner() else {
        return;
    };
    let mut plan = domain("tree_support_tip_diameter");
    plan.cases = [
        ("legal-min", "0.1"),
        ("legal-max", "10"),
        ("rejected-max", "100"),
    ]
    .map(|(label, value)| domains::OptionCase {
        label: label.into(),
        value: Some(Value::from(value)),
    })
    .into();
    let moves = compare_cases(&runner, &plan, &[("support_style", "organic")]);
    assert!(moves["legal-min"] != moves["legal-max"]);
    assert_eq!(
        moves.len(),
        2,
        "native rejection is not a complete output pair"
    );
}

fn compare_domain(key: &str) -> Option<BTreeMap<String, Vec<String>>> {
    let runner = reference_runner()?;
    Some(compare_cases(&runner, &domain(key), &[]))
}

fn reference_runner() -> Option<OrcaRunner> {
    if std::env::var("ARES_PARITY_TREE_SUPPORT_REFERENCE").as_deref() != Ok("1") {
        return None;
    }
    Some(OrcaRunner::from_env().unwrap().unwrap())
}

fn domain(key: &str) -> domains::OptionPlan {
    domains::load(&runner::repo_root())
        .into_iter()
        .find(|plan| plan.key == key)
        .unwrap()
}

fn compare_cases(
    runner: &OrcaRunner,
    plan: &domains::OptionPlan,
    overrides: &[(&str, &str)],
) -> BTreeMap<String, Vec<String>> {
    let key = &plan.key;
    let profiles = VendorProfiles::load(&profiles_root(), "Creality").unwrap();
    let selection =
        parity::select_printer(&profiles, "Creality", "Creality Ender-3 0.4 nozzle").unwrap();
    let mut movements = BTreeMap::new();
    let mut failures = Vec::new();
    for case in &plan.cases {
        let mut machine = profiles.machine(&selection.printer).unwrap();
        let mut process = profiles.process(&selection.process).unwrap();
        parity::normalize_process_defaults(&machine, &mut process);
        let mut filaments = selection
            .filaments
            .iter()
            .map(|name| profiles.filament(name).unwrap())
            .collect::<Vec<_>>();
        parity::normalize_filament_defaults(&mut filaments);
        inject_case(plan, case, &mut machine, &mut process, &mut filaments);
        for (key, value) in overrides {
            process.insert((*key).into(), Value::from(*value));
        }
        let style = process["support_style"].as_str().unwrap();
        let label = format!("tree-activation/{key}/{style}/{}", case.label);
        let exported = runner
            .export_case(
                &CaseInputs {
                    label: &label,
                    machine: &machine,
                    process: &process,
                    filaments: &filaments,
                },
                &parity::smoke_overrides(),
                &runner::repo_root().join("tests/parity/overhang.stl"),
            )
            .unwrap();
        runner::application::verify_value(&exported.project, key, case.value.as_ref().unwrap())
            .unwrap();
        for dependency in ["enable_support", "support_type", "support_style"] {
            runner::application::verify_value(&exported.project, dependency, &process[dependency])
                .unwrap();
        }
        let dependencies: &[(&str, &str)] = match key.as_str() {
            "tree_support_branch_distance" => &[("max_bridge_length", "2")],
            "tree_support_angle_slow" => &[("tree_support_branch_angle_organic", "60")],
            "tree_support_tip_diameter" => &[
                ("tree_support_branch_diameter_organic", "10"),
                ("support_line_width", "0.1"),
                ("layer_height", "0.08"),
                ("initial_layer_print_height", "0.08"),
            ],
            "tree_support_auto_brim" => &[("raft_layers", "0")],
            "tree_support_brim_width" => &[("raft_layers", "0"), ("tree_support_auto_brim", "0")],
            _ => &[],
        };
        for (dependency, value) in dependencies {
            runner::application::verify_value(&exported.project, dependency, &Value::from(*value))
                .unwrap();
        }
        if case.label == "rejected-max" {
            let Err(error) = runner.slice_case(&exported) else {
                panic!("explicit Organic tip100 must fail native validation");
            };
            let captured = error.captured.as_ref().unwrap();
            assert_eq!(captured.code, Some(if cfg!(unix) { 205 } else { -51 }));
            assert_eq!(
                std::str::from_utf8(&captured.stderr).unwrap().trim(),
                "Organic support branch diameter must not be smaller than support tree tip diameter."
            );
            eprintln!(
                "{label}: native-only rejection evidence {:?}",
                error.evidence
            );
            continue;
        }
        let sliced = runner.slice_case(&exported).unwrap();
        let reference = std::str::from_utf8(&sliced.reference).unwrap();
        assert!(
            normal_support_output::has_support_extrusion(reference),
            "{label}"
        );
        movements.insert(case.label.clone(), support_moves(reference));
        let outcome = parity::compare_case(&sliced);
        if outcome.status != "PASS" {
            failures.push(format!("{label}: {}", outcome.detail));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    movements
}

fn support_moves(gcode: &str) -> Vec<String> {
    let mut support = false;
    let mut relative_e = false;
    gcode
        .lines()
        .filter_map(|line| {
            if let Some(role) = line.strip_prefix(";TYPE:") {
                support = matches!(role, "Support" | "Support interface");
            }
            let instruction = line.split(';').next().unwrap();
            let words = instruction.split_ascii_whitespace();
            match words.clone().next() {
                Some("M83") => relative_e = true,
                Some("M82") => relative_e = false,
                _ => {}
            }
            (relative_e
                && support
                && instruction.starts_with("G1 ")
                && words.clone().any(|word| word.starts_with(['X', 'Y']))
                && words.clone().any(|word| {
                    word.strip_prefix('E')
                        .is_some_and(|value| value.parse::<f64>().unwrap() > 0.0)
                }))
            .then(|| {
                words
                    .filter(|word| word.starts_with(['X', 'Y']))
                    .collect::<Vec<_>>()
                    .join(" ")
            })
        })
        .collect()
}
