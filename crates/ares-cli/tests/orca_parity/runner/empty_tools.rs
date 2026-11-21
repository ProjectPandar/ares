//! `GCode.cpp::process_layer`: retained final geometry with no extrusion tools.
use super::{CaseInputs, OrcaRunner, application, repo_root};
use crate::{self as parity, presets::VendorProfiles};
use serde_json::json;

#[test]
fn final_empty_geometry_layer_preserves_complete_output() {
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
    let machine = profiles.machine(&selection.printer).unwrap();
    let mut process = profiles.process(&selection.process).unwrap();
    parity::normalize_process_defaults(&machine, &mut process);
    let settings = [
        ("enable_support", "0"),
        ("wall_loops", "0"),
        ("top_shell_layers", "0"),
        ("top_shell_thickness", "0"),
        ("bottom_shell_layers", "3"),
        ("bottom_shell_thickness", "0"),
        ("sparse_infill_density", "0%"),
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
    for slope in ["0", "2"] {
        process.insert("max_volumetric_extrusion_rate_slope".into(), json!(slope));
        let label = format!("layer-tools/final-empty/equalizer-{slope}");
        let exported = runner
            .export_case(
                &CaseInputs {
                    label: &label,
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
            "max_volumetric_extrusion_rate_slope",
            &json!(slope),
        )
        .unwrap();
        let sliced = runner.slice_case(&exported).unwrap();
        let outcome = parity::compare_case(&sliced);
        let native = std::str::from_utf8(&sliced.reference).unwrap();
        assert_eq!(
            native
                .lines()
                .filter(|line| *line == ";LAYER_CHANGE")
                .count(),
            3
        );
        assert!(native.contains("; total layers count = 50\n"));
        assert_eq!(outcome.status, "PASS", "{}", outcome.detail);
    }
}
