#[path = "monotonic_chain/fixture.rs"]
mod fixture;

use super::*;
use crate::runner::CaseInputs;
use serde_json::json;

#[test]
fn concave_monotonic_fill_preserves_complete_output() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let profiles = VendorProfiles::load(&profiles_root(), "Creality").unwrap();
    let selection =
        parity::select_printer(&profiles, "Creality", "Creality Ender-3 0.4 nozzle").unwrap();
    let machine = profiles.machine(&selection.printer).unwrap();
    let mut process = profiles.process(&selection.process).unwrap();
    parity::normalize_process_defaults(&machine, &mut process);
    let settings = [
        ("detect_narrow_internal_solid_infill", "0"),
        ("enable_prime_tower", "0"),
        ("wall_loops", "0"),
        ("skirt_loops", "0"),
        ("elefant_foot_compensation", "0"),
        ("infill_direction", "45"),
        ("solid_infill_direction", "45"),
        ("align_infill_direction_to_model", "1"),
        ("bottom_surface_pattern", "monotonic"),
        ("top_surface_pattern", "monotonic"),
        ("layer_height", "0.2"),
        ("initial_layer_print_height", "0.2"),
        ("line_width", "0.42"),
        ("initial_layer_line_width", "0.42"),
        ("top_surface_line_width", "0.42"),
        ("internal_solid_infill_line_width", "0.42"),
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
    let directory = tempfile::tempdir().unwrap();
    let model = directory.path().join("concave-slab.stl");
    std::fs::write(&model, fixture::concave_slab()).unwrap();
    let exported = runner
        .export_case(
            &CaseInputs {
                label: "monotonic-chain/concave-slab",
                machine: &machine,
                process: &process,
                filaments: &filaments,
            },
            &parity::smoke_overrides(),
            &model,
        )
        .unwrap();
    for (key, value) in settings {
        runner::application::verify_value(&exported.project, key, &json!(value)).unwrap();
    }
    let sliced = runner.slice_case(&exported).unwrap();
    assert!(
        std::str::from_utf8(&sliced.reference)
            .unwrap()
            .contains(";TYPE:Bottom surface\n")
    );
    let outcome = parity::compare_case(&sliced);
    assert_eq!(outcome.status, "PASS", "{}", outcome.detail);
}
