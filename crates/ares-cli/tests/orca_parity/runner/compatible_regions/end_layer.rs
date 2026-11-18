use super::{CaseInputs, OrcaRunner, application, parity, repo_root};
use crate::presets::VendorProfiles;
use serde_json::json;

#[test]
fn machine_end_uses_live_integer_layer_num_in_complete_output() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let profiles = VendorProfiles::load(
        &repo_root().join("OrcaSlicer/resources/profiles"),
        "Creality",
    )
    .unwrap();
    let selection =
        parity::select_printer(&profiles, "Creality", "Creality Ender-3 0.4 nozzle").unwrap();
    let mut machine = profiles.machine(&selection.printer).unwrap();
    let machine_end = format!(
        "{}\n; parity final layer={{layer_num}} half={{layer_num / 2}}\n",
        machine["machine_end_gcode"].as_str().unwrap()
    );
    machine.insert("machine_end_gcode".into(), json!(machine_end));
    let mut process = profiles.process(&selection.process).unwrap();
    parity::normalize_process_defaults(&machine, &mut process);
    let settings = [
        ("enable_support", "0"),
        ("enable_prime_tower", "0"),
        ("layer_height", "0.2"),
        ("initial_layer_print_height", "0.2"),
    ];
    for (key, value) in settings {
        process.insert(key.into(), json!(value));
    }
    let mut filaments = selection
        .filaments
        .iter()
        .map(|name| profiles.filament(name).unwrap())
        .collect::<Vec<_>>();
    parity::normalize_filament_defaults(&mut filaments);
    let exported = runner
        .export_case(
            &CaseInputs {
                label: "compatible-regions/live-integer-end-layer",
                machine: &machine,
                process: &process,
                filaments: &filaments,
            },
            &parity::smoke_overrides(),
            &repo_root().join("tests/parity/cube10.stl"),
        )
        .unwrap();
    application::verify_value(&exported.project, "machine_end_gcode", &json!(machine_end)).unwrap();
    for (key, value) in settings {
        application::verify_value(&exported.project, key, &json!(value)).unwrap();
    }
    let case = runner.slice_case(&exported).unwrap();
    let native = std::str::from_utf8(&case.reference).unwrap();
    assert!(
        native
            .lines()
            .any(|line| line == "; parity final layer=49 half=24")
    );
    let outcome = parity::compare_case(&case);
    assert_eq!(outcome.status, "PASS", "{}", outcome.detail);
}
