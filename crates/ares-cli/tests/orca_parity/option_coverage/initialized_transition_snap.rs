#[path = "initialized_transition_snap/model.rs"]
mod model;

use super::*;
use crate::runner::CaseInputs;

#[test]
fn initialized_transition_snap_matches_complete_internal_fill() {
    if !crate::runner::seeded_reference_enabled(Some("ARES_PARITY_TRANSITION_SNAP_REFERENCE")) {
        return;
    }
    let runner = OrcaRunner::from_env().unwrap().unwrap();
    assert_eq!(std::env::var("ORCA_REPLAY_SEED").unwrap(), "0");
    let directory = tempfile::tempdir().unwrap();
    let model = directory.path().join("transition-snap.stl");
    std::fs::write(&model, model::prism()).unwrap();
    let profiles = VendorProfiles::load(&profiles_root(), "Creality").unwrap();
    let selection =
        parity::select_printer(&profiles, "Creality", "Creality Ender-3 0.4 nozzle").unwrap();
    let machine = profiles.machine(&selection.printer).unwrap();
    let mut process = profiles.process(&selection.process).unwrap();
    parity::normalize_process_defaults(&machine, &mut process);
    let settings = [
        ("wall_loops", "0"),
        ("layer_height", "0.2"),
        ("initial_layer_print_height", "0.2"),
        ("sparse_infill_density", "100%"),
        ("top_shell_layers", "0"),
        ("bottom_shell_layers", "0"),
        ("detect_narrow_internal_solid_infill", "1"),
        ("internal_solid_infill_line_width", "0.42"),
        ("enable_support", "0"),
        ("disable_m73", "1"),
        ("enable_prime_tower", "0"),
    ];
    for (key, value) in settings {
        process.insert(key.into(), Value::String(value.into()));
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
                label: "initialized-transition-snap/internal-fill/no-m73",
                machine: &machine,
                process: &process,
                filaments: &filaments,
            },
            &parity::smoke_overrides(),
            &model,
        )
        .unwrap();
    for (key, value) in settings {
        runner::application::verify_value(&exported.project, key, &Value::String(value.into()))
            .unwrap();
    }
    let sliced = runner.slice_case(&exported).unwrap();
    let reference = std::str::from_utf8(&sliced.reference).unwrap();
    assert!(reference.contains(";TYPE:Internal solid infill"));
    assert!(!reference.contains("\nM73 "));
    let outcome = parity::compare_case(&sliced);
    assert_eq!(outcome.status, "PASS", "{}", outcome.detail);
}
