use super::*;
use crate::runner::CaseInputs;

#[test]
fn slow_layer_domain_covers_scalar_endpoints_seed_and_short_ramp() {
    let Some(plans) = domains::try_load(&runner::repo_root()) else {
        return;
    };
    let plan = plans.iter().find(|p| p.key == "slow_down_layers").unwrap();
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
    assert_eq!(values.len(), 5);
    assert_eq!(&values[..2], &[0, i32::MAX]);
    assert!((1..i32::MAX).contains(&values[2]));
    assert_eq!(&values[3..], &[1, 4]);
    assert!(plan.provably_legal);
    assert!(plan.omission.is_none());
}

#[test]
fn orca_parity_slow_layer_counts_match_complete_output_and_activate_motion() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let plan = domains::load(&runner::repo_root())
        .into_iter()
        .find(|p| p.key == "slow_down_layers")
        .unwrap();
    let profiles = VendorProfiles::load(&profiles_root(), "Creality").unwrap();
    let selection =
        parity::select_printer(&profiles, "Creality", "Creality Ender-3 0.4 nozzle").unwrap();
    let mut disabled_motion = Vec::new();
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
        inject_case(&plan, case, &mut machine, &mut process, &mut filaments);
        let requested = case.value.as_ref().unwrap();
        let value = requested.as_str().unwrap();
        let exported = runner
            .export_case(
                &CaseInputs {
                    label: &format!("slow-layer-count/{value}"),
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
        let proof =
            runner::application::verify_value(&exported.project, &plan.key, requested).unwrap();
        assert_eq!(proof["requested"], *requested);
        assert_eq!(proof["effective"], *requested);
        let sliced = runner.slice_case(&exported).unwrap();
        let reference = std::str::from_utf8(&sliced.reference).unwrap();
        assert!(
            reference
                .lines()
                .any(|line| line == format!("; slow_down_layers = {value}"))
        );
        let outcome = parity::compare_case(&sliced);
        assert_eq!(outcome.status, "PASS", "{value}: {}", outcome.detail);
        let motion = reference
            .lines()
            .filter(|line| line.starts_with("G1 ") && line.contains(" F"))
            .map(str::to_owned)
            .collect::<Vec<_>>();
        match value {
            "0" => disabled_motion = motion,
            "1" => assert_eq!(motion, disabled_motion),
            _ => assert_ne!(motion, disabled_motion, "{value} must activate the ramp"),
        }
    }
}
