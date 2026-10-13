use super::*;

#[test]
fn orca_parity_filament_layer_retraction_preserves_every_value_and_complete_output() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
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
    let plan = domains::load(&runner::repo_root())
        .into_iter()
        .find(|plan| plan.key == "filament_retract_when_changing_layer")
        .unwrap();
    assert_eq!(
        plan.cases
            .iter()
            .map(|case| case.label.as_str())
            .collect::<Vec<_>>(),
        ["false", "true", "nil"]
    );
    let outcome = execute_plan(&runner, &plan, &machine, &process, &filaments);
    assert_eq!(outcome.status, "PASS", "{}", outcome.detail);
    assert_eq!(outcome.compared, 3);
}

#[test]
fn orca_parity_wall_sequence_keeps_each_effective_value_and_full_output() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
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
    let mut plan = domains::load(&runner::repo_root())
        .into_iter()
        .find(|plan| plan.key == "wall_sequence")
        .unwrap();
    let outcome = execute_plan(&runner, &plan, &machine, &process, &filaments);
    assert_eq!(outcome.status, "PASS", "{}", outcome.detail);
    assert_eq!(outcome.compared, 3);

    let root = parity::artifacts::root_from_env().unwrap();
    let proofs = wall_proofs(&root);
    let mut requested = proofs.values().cloned().collect::<Vec<_>>();
    requested.sort();
    assert_eq!(
        requested,
        [
            "inner wall/outer wall",
            "inner-outer-inner wall",
            "outer wall/inner wall"
        ]
    );

    plan.cases
        .retain(|case| case.label == "outer wall/inner wall");
    plan.provably_legal = false;
    let filtered = execute_plan(&runner, &plan, &machine, &process, &filaments);
    assert_eq!(filtered.compared, 1, "{}", filtered.detail);
    assert_eq!(filtered.status, "INCOMPLETE");
    assert_eq!(wall_proofs(&root), proofs);
}

fn wall_proofs(root: &std::path::Path) -> std::collections::BTreeMap<PathBuf, String> {
    std::fs::read_dir(root)
        .unwrap()
        .map(Result::unwrap)
        .filter(|entry| {
            let name = entry.file_name();
            let name = name.to_str().unwrap();
            name.starts_with("wall_sequence-") && name.ends_with("-application.json")
        })
        .map(|entry| {
            assert!(entry.file_type().unwrap().is_file());
            let proof: Value =
                serde_json::from_slice(&std::fs::read(entry.path()).unwrap()).unwrap();
            assert_eq!(proof["key"], "wall_sequence");
            assert_eq!(proof["requested"], proof["effective"]);
            (
                entry.path(),
                proof["requested"].as_str().unwrap().to_owned(),
            )
        })
        .collect()
}

// PrintConfig.cpp:5879-5894: both enum values need successful full-stream
// coverage, not merely a legal schema value followed by Native rejection.
#[test]
fn orca_parity_timelapse_type_compares_both_values_as_complete_domain() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
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
    let plan = domains::load(&runner::repo_root())
        .into_iter()
        .find(|plan| plan.key == "timelapse_type")
        .unwrap();
    assert_eq!(
        plan.cases
            .iter()
            .map(|case| case.label.as_str())
            .collect::<Vec<_>>(),
        ["0", "1"]
    );
    let outcome = execute_plan(&runner, &plan, &machine, &process, &filaments);
    assert_eq!(outcome.status, "PASS", "{}", outcome.detail);
    assert_eq!(outcome.compared, 2, "{}", outcome.detail);
}

// PrintConfig.cpp:5405,9004-9055,9634ff; PrintApply.cpp:1169.
#[test]
fn orca_parity_print_extruder_id_accounts_six_materialized_identity_cases() {
    let runner = OrcaRunner::from_env()
        .unwrap()
        .expect("print identity matrix requires the live Native oracle");
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
    let plan = domains::load(&runner::repo_root())
        .into_iter()
        .find(|plan| plan.key == "print_extruder_id")
        .unwrap();
    let outcome = execute_plan(&runner, &plan, &machine, &process, &filaments);
    write_summary(std::slice::from_ref(&outcome));
    assert_eq!(outcome.status, "PASS", "{}", outcome.detail);
    assert_eq!(outcome.cases, 6);
    assert_eq!(outcome.compared, 6);
    assert_eq!(outcome.rejected, 0);
    assert!(plan.cases.iter().all(|case| case.value.is_some()));
    let expected = [
        "reversed",
        "ordered",
        "default",
        "repeated",
        "short",
        "unmatched",
    ];
    assert_eq!(
        plan.cases
            .iter()
            .map(|case| case.label.as_str())
            .collect::<Vec<_>>(),
        expected
    );
    let root = parity::artifacts::root_from_env().unwrap();
    for label in expected {
        let receipt: Value = serde_json::from_slice(
            &std::fs::read(root.join(format!("print-extruder-id-{label}-comparison.json")))
                .unwrap(),
        )
        .unwrap();
        assert_eq!(receipt["key"], "print_extruder_id");
        assert_eq!(receipt["label"], label);
        assert_eq!(receipt["comparison"], "PASS");
        for member in [
            "input.3mf",
            "native.gcode",
            "ares.gcode",
            "requested-settings.json",
        ] {
            assert!(
                PathBuf::from(receipt["directory"].as_str().unwrap())
                    .join(member)
                    .is_file()
            );
        }
    }
}
