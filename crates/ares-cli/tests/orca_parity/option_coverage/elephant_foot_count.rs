use super::*;
use crate::runner::CaseInputs;

#[test]
fn elephant_foot_domain_covers_scalar_endpoints_and_full_range_seed() {
    let Some(plans) = domains::try_load(&runner::repo_root()) else {
        return;
    };
    let plan = plans
        .iter()
        .find(|p| p.key == "elefant_foot_compensation_layers")
        .unwrap();
    let values = plan
        .cases
        .iter()
        .map(|case| {
            case.value
                .as_ref()
                .unwrap()
                .as_str()
                .unwrap()
                .parse::<i32>()
                .unwrap()
        })
        .collect::<Vec<_>>();
    assert_eq!(values.len(), 3);
    assert_eq!(&values[..2], &[1, i32::MAX]);
    assert!((2..i32::MAX).contains(&values[2]));
    assert!(plan.provably_legal);
    assert!(plan.omission.is_none());
}

#[test]
fn orca_parity_elephant_foot_counts_match_complete_output_and_activate_geometry() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let plan = domains::load(&runner::repo_root())
        .into_iter()
        .find(|p| p.key == "elefant_foot_compensation_layers")
        .unwrap();
    let profiles = VendorProfiles::load(&profiles_root(), "Creality").unwrap();
    let selection =
        parity::select_printer(&profiles, "Creality", "Creality Ender-3 0.4 nozzle").unwrap();
    let control = domains::OptionCase {
        label: "disabled".to_owned(),
        value: Some(Value::String("1".to_owned())),
    };
    let mut disabled_motion = Vec::new();
    let mut single_layer_motion = Vec::new();
    let mut failures = Vec::new();
    for (active, case) in
        std::iter::once((false, &control)).chain(plan.cases.iter().map(|case| (true, case)))
    {
        let mut machine = profiles.machine(&selection.printer).unwrap();
        let mut process = profiles.process(&selection.process).unwrap();
        parity::normalize_process_defaults(&machine, &mut process);
        let mut filaments = selection
            .filaments
            .iter()
            .map(|name| profiles.filament(name).unwrap())
            .collect::<Vec<_>>();
        parity::normalize_filament_defaults(&mut filaments);
        inject_case(&plan, case, &mut machine, &mut process, &mut filaments);
        if !active {
            process.insert(
                "elefant_foot_compensation".to_owned(),
                Value::String("0".to_owned()),
            );
        }
        let requested = case.value.as_ref().unwrap();
        let value = requested.as_str().unwrap();
        let exported = runner
            .export_case(
                &CaseInputs {
                    label: &format!("elephant-foot-count/{}", case.label),
                    machine: &machine,
                    process: &process,
                    filaments: &filaments,
                },
                &parity::smoke_overrides(),
                &runner::repo_root().join("tests/parity/cube10.stl"),
            )
            .unwrap();
        let settings = runner::application::settings(&exported.project).unwrap();
        assert_eq!(settings["raft_layers"], "0");
        assert_eq!(settings["elefant_foot_layers_density"], "100%");
        assert_eq!(
            settings["elefant_foot_compensation"],
            if active { "0.15" } else { "0" }
        );
        let proof =
            runner::application::verify_value(&exported.project, &plan.key, requested).unwrap();
        assert_eq!(proof["requested"], *requested);
        assert_eq!(proof["effective"], *requested);
        let sliced = runner.slice_case(&exported).unwrap();
        let reference = std::str::from_utf8(&sliced.reference).unwrap();
        assert!(
            reference
                .lines()
                .any(|line| { line == format!("; elefant_foot_compensation_layers = {value}") })
        );
        let outcome = parity::compare_case(&sliced);
        if outcome.status != "PASS" {
            failures.push(format!("{}: {}", case.label, outcome.detail));
        }
        let motion = reference
            .lines()
            .filter(|line| line.starts_with("G1 "))
            .map(str::to_owned)
            .collect::<Vec<_>>();
        if !active {
            disabled_motion = motion;
        } else {
            assert_ne!(
                motion, disabled_motion,
                "{value} must activate compensation"
            );
            if value == "1" {
                single_layer_motion = motion;
            } else {
                assert_ne!(
                    motion, single_layer_motion,
                    "{value} must extend compensation"
                );
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
