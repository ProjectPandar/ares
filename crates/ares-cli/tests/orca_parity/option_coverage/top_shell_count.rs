use super::*;
use crate::runner::CaseInputs;

#[test]
fn top_shell_domain_covers_scalar_endpoints_and_full_range_seed() {
    let Some(plans) = domains::try_load(&runner::repo_root()) else {
        return;
    };
    let plan = plans.iter().find(|p| p.key == "top_shell_layers").unwrap();
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
    assert_eq!(&values[..2], &[0, i32::MAX]);
    assert!((1..i32::MAX).contains(&values[2]));
    assert!(plan.provably_legal);
    assert!(plan.omission.is_none());
}

#[test]
fn orca_parity_top_shell_counts_match_complete_non_spiral_output() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let plan = domains::load(&runner::repo_root())
        .into_iter()
        .find(|p| p.key == "top_shell_layers")
        .unwrap();
    let profiles = VendorProfiles::load(&profiles_root(), "Creality").unwrap();
    let selection =
        parity::select_printer(&profiles, "Creality", "Creality Ender-3 0.4 nozzle").unwrap();
    let mut cases = plan.cases.clone();
    cases.extend(
        [i32::MAX - 100, i32::MAX - 1].map(|value| domains::OptionCase {
            label: value.to_string(),
            value: Some(Value::String(value.to_string())),
        }),
    );
    let mut failures = Vec::new();
    for case in &cases {
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
                    label: &format!("top-shell-count/{value}"),
                    machine: &machine,
                    process: &process,
                    filaments: &filaments,
                },
                &parity::smoke_overrides(),
                &runner::repo_root().join("tests/parity/cube10.stl"),
            )
            .unwrap();
        let settings = runner::application::settings(&exported.project).unwrap();
        assert_eq!(settings["spiral_mode"], "0");
        assert_eq!(settings["ensure_vertical_shell_thickness"], "ensure_all");
        assert_eq!(settings["top_shell_thickness"], "0.8");
        let proof =
            runner::application::verify_value(&exported.project, &plan.key, requested).unwrap();
        assert_eq!(proof["requested"], *requested);
        assert_eq!(proof["effective"], *requested);
        let sliced = runner.slice_case(&exported).unwrap();
        let reference = std::str::from_utf8(&sliced.reference).unwrap();
        assert!(
            reference
                .lines()
                .any(|line| line == format!("; top_shell_layers = {value}"))
        );
        let outcome = parity::compare_case(&sliced);
        if outcome.status != "PASS" {
            failures.push(format!("{value}: {}", outcome.detail));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
