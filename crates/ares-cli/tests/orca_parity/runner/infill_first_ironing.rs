use super::{CaseInputs, OrcaRunner, application, repo_root};
use crate::{self as parity, presets::VendorProfiles};
use serde_json::json;

#[test]
fn infill_first_ironing_follows_walls_in_complete_output() {
    assert_complete_output("");
}

#[test]
fn infill_first_ironing_preserves_traditional_timelapse() {
    assert_complete_output("M400 ; emission-phase-timelapse");
}

fn assert_complete_output(time_lapse_gcode: &str) {
    if !crate::runner::reference_enabled("ARES_PARITY_EMISSION_PHASES_REFERENCE") {
        return;
    }
    let runner = OrcaRunner::from_env().unwrap().unwrap();
    let profiles = VendorProfiles::load(
        &repo_root().join("OrcaSlicer/resources/profiles"),
        "Creality",
    )
    .unwrap();
    let selection =
        parity::select_printer(&profiles, "Creality", "Creality Ender-3 0.4 nozzle").unwrap();
    let mut machine = profiles.machine(&selection.printer).unwrap();
    machine.insert("time_lapse_gcode".into(), json!(time_lapse_gcode));
    let mut process = profiles.process(&selection.process).unwrap();
    parity::normalize_process_defaults(&machine, &mut process);
    let settings = [
        ("enable_support", "0"),
        ("is_infill_first", "1"),
        ("ironing_type", "top"),
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
                label: &format!("emission-phases/infill-first-ironing/{time_lapse_gcode}"),
                machine: &machine,
                process: &process,
                filaments: &filaments,
            },
            &parity::smoke_overrides(),
            &repo_root().join("tests/parity/cube10.stl"),
        )
        .unwrap();
    for (key, value) in settings {
        application::verify_value(&exported.project, key, &json!(value)).unwrap();
    }
    application::verify_value(
        &exported.project,
        "time_lapse_gcode",
        &json!(time_lapse_gcode),
    )
    .unwrap();
    let sliced = runner.slice_case(&exported).unwrap();
    let outcome = parity::compare_case(&sliced);
    let native = std::str::from_utf8(&sliced.reference).unwrap();
    let last_layer = native.rsplit(";LAYER_CHANGE").next().unwrap();
    let inner = last_layer.find(";TYPE:Inner wall").unwrap();
    let ironing = last_layer.find(";TYPE:Ironing").unwrap();
    assert!(inner < ironing);
    if !time_lapse_gcode.is_empty() {
        let marker = last_layer.find(time_lapse_gcode).unwrap();
        let infill = last_layer.find(";TYPE:Top surface").unwrap();
        assert!(marker < infill && infill < inner);
    }
    assert_eq!(outcome.status, "PASS", "{}", outcome.detail);
}
