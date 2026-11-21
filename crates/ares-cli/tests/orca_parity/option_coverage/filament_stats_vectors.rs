use super::*;
use crate::runner::CaseInputs;

#[test]
fn postprocessed_statistics_include_unused_trailing_filament() {
    if !crate::runner::reference_available() {
        return;
    }
    for cost in ["25", "0"] {
        compare_statistics(cost);
    }
}

fn compare_statistics(cost: &str) {
    let runner = OrcaRunner::from_env().unwrap().unwrap();
    let profiles = VendorProfiles::load(&profiles_root(), "BBL").unwrap();
    let selection = parity::select_printer(&profiles, "BBL", "Bambu Lab H2D 0.4 nozzle").unwrap();
    let mut machine = profiles.machine(&selection.printer).unwrap();
    machine.insert("filament_map_mode".into(), Value::String("Manual".into()));
    machine.insert("filament_map".into(), serde_json::json!(["1", "1"]));
    let mut process = profiles.process(&selection.process).unwrap();
    parity::normalize_process_defaults(&machine, &mut process);
    process.insert("enable_prime_tower".into(), Value::String("0".into()));
    process.insert("disable_m73".into(), Value::String("1".into()));
    let mut filaments = vec![profiles.filament(&selection.filaments[0]).unwrap(); 2];
    parity::normalize_filament_defaults(&mut filaments);
    for (filament, colour) in filaments.iter_mut().zip(["#F2754E", "#0000FF"]) {
        filament.insert("filament_colour".into(), serde_json::json!([colour]));
        filament.insert("filament_density".into(), serde_json::json!(["1.24"]));
        filament.insert("filament_cost".into(), serde_json::json!([cost]));
    }
    let label = format!("filament-statistics/unused-trailing-filament/cost-{cost}");
    let exported = runner
        .export_case(
            &CaseInputs {
                label: &label,
                machine: &machine,
                process: &process,
                filaments: &filaments,
            },
            &parity::smoke_overrides(),
            &runner::repo_root().join("tests/parity/cube10.stl"),
        )
        .unwrap();
    let exported = runner
        .bind_filament_map_mode(exported, &Value::String("Manual".into()))
        .unwrap();
    runner::filament_map::verify(&exported.project, &Value::String("Manual".into())).unwrap();
    let settings = runner::application::settings(&exported.project).unwrap();
    assert_eq!(settings["filament_diameter"].as_array().unwrap().len(), 2);
    assert_eq!(
        settings["filament_density"],
        serde_json::json!(["1.24", "1.24"])
    );
    assert_eq!(settings["filament_cost"], serde_json::json!([cost, cost]));
    let sliced = runner.slice_case(&exported).unwrap();
    let native = std::str::from_utf8(&sliced.reference).unwrap();
    for prefix in ["; filament used [g] = ", "; filament cost = "] {
        let values = native.lines().find_map(|line| line.strip_prefix(prefix));
        if prefix == "; filament cost = " && cost == "0" {
            assert!(values.is_none());
        } else {
            let values = values.unwrap().split(", ").collect::<Vec<_>>();
            assert_eq!(values.len(), 2);
            assert!(values[0].parse::<f64>().unwrap() > 0.0);
            assert_eq!(values[1], "0.00");
        }
    }
    let outcome = parity::compare_case(&sliced);
    assert_eq!(outcome.status, "PASS", "{}", outcome.detail);
}
