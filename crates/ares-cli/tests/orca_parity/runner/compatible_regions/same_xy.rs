use super::{CaseInputs, OrcaRunner, application, parity, repo_root};
use crate::presets::VendorProfiles;
use serde_json::json;

#[test]
fn thin_square_travel_preserves_complete_native_output() {
    compare("none");
}

#[test]
fn thin_square_scarf_travel_preserves_complete_native_output() {
    compare("external");
}

fn compare(scarf: &str) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let profiles = VendorProfiles::load(
        &repo_root().join("OrcaSlicer/resources/profiles"),
        "Creality",
    )
    .unwrap();
    let selection =
        parity::select_printer(&profiles, "Creality", "Creality Ender-3 V3 SE 0.2 nozzle").unwrap();
    let machine = profiles.machine(&selection.printer).unwrap();
    let mut process = profiles.process(&selection.process).unwrap();
    parity::normalize_process_defaults(&machine, &mut process);
    let settings = [
        ("enable_support", "0"),
        ("enable_prime_tower", "0"),
        ("sparse_infill_density", "0%"),
        ("layer_height", "0.14"),
        ("line_width", "0.22"),
        ("outer_wall_line_width", "0.22"),
        ("inner_wall_line_width", "0.22"),
        ("wall_loops", "2"),
        ("reduce_crossing_wall", "1"),
        ("max_travel_detour_distance", "0"),
        ("seam_slope_type", scarf),
        ("seam_slope_conditional", "0"),
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
                label: "compatible-regions/thin-square-travel",
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
    let mut case = runner.slice_case(&exported).unwrap();
    case.label.push_str(scarf);
    let outcome = parity::compare_case(&case);
    assert_eq!(outcome.status, "PASS", "{}", outcome.detail);
}
