use super::*;
use crate::runner::CaseInputs;
use serde_json::json;

#[test]
fn orca_parity_filament_ironing_speed_has_active_paths() {
    let Some(runner) = OrcaRunner::from_env().unwrap_or_else(|e| panic!("{e}")) else {
        return;
    };
    let plan = domains::load(&runner::repo_root())
        .into_iter()
        .find(|plan| plan.key == "filament_ironing_speed")
        .unwrap();
    let profiles = VendorProfiles::load(&profiles_root(), "Creality").unwrap();
    let selection =
        parity::select_printer(&profiles, "Creality", "Creality Ender-3 0.4 nozzle").unwrap();
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
        let exported = runner
            .export_case(
                &CaseInputs {
                    label: &format!("filament-ironing-speed/{}", case.label),
                    machine: &machine,
                    process: &process,
                    filaments: &filaments,
                },
                &parity::smoke_overrides(),
                &runner::repo_root().join("tests/parity/cube10.stl"),
            )
            .unwrap();
        if let Some(value) = &case.value {
            runner::application::verify_value(&exported.project, &plan.key, value).unwrap();
        }
        runner::application::verify_value(&exported.project, "ironing_type", &json!("top"))
            .unwrap();
        let sliced = runner.slice_case(&exported).unwrap();
        assert!(has_ironing_extrusion(
            std::str::from_utf8(&sliced.reference).unwrap()
        ));
        let outcome = parity::compare_case(&sliced);
        assert_eq!(outcome.status, "PASS", "{}: {}", case.label, outcome.detail);
    }
}

fn has_ironing_extrusion(reference: &str) -> bool {
    let mut ironing = false;
    let mut relative = false;
    for line in reference.lines() {
        if let Some(role) = line.strip_prefix(";TYPE:") {
            ironing = role == "Ironing";
        }
        let mut words = line.split(';').next().unwrap().split_ascii_whitespace();
        match words.clone().next() {
            Some("M83") => relative = true,
            Some("M82") => relative = false,
            Some("G1")
                if ironing
                    && relative
                    && words.clone().any(|word| word.starts_with(['X', 'Y'])) =>
            {
                if let Some(e) = words.find_map(|word| {
                    word.strip_prefix('E')
                        .map(|value| value.parse::<f64>().unwrap())
                }) && e > 0.0
                {
                    return true;
                }
            }
            _ => {}
        }
    }
    false
}
