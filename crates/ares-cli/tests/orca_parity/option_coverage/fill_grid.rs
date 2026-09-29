use super::*;
use crate::runner::CaseInputs;
use serde_json::json;

#[test]
fn orca_parity_wide_solid_fill_preserves_complete_output() {
    compare("fill-grid/wide-solid", None, false);
}

#[test]
fn orca_parity_signed_cross_zag_shifts_preserve_complete_output() {
    for step in ["0.731272", "2.30772"] {
        compare(&format!("fill-grid/cross-zag-{step}"), Some(step), false);
    }
}

#[test]
fn orca_parity_locked_zag_shifts_preserve_complete_output() {
    compare("fill-grid/locked-zag", Some("0.731272"), true);
}

fn compare(label: &str, shift: Option<&str>, locked: bool) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let profiles = VendorProfiles::load(&profiles_root(), "Creality").unwrap();
    let selection =
        parity::select_printer(&profiles, "Creality", "Creality Ender-3 0.4 nozzle").unwrap();
    let machine = profiles.machine(&selection.printer).unwrap();
    let mut process = profiles.process(&selection.process).unwrap();
    parity::normalize_process_defaults(&machine, &mut process);
    let mut settings = vec![
        ("detect_narrow_internal_solid_infill", "0"),
        ("enable_prime_tower", "0"),
        ("wall_loops", "0"),
        ("infill_direction", "45"),
        ("bottom_surface_pattern", "monotonic"),
    ];
    if let Some(step) = shift {
        settings.extend([
            (
                "sparse_infill_pattern",
                if locked { "lockedzag" } else { "crosszag" },
            ),
            ("sparse_infill_density", "40%"),
            ("infill_shift_step", step),
            ("top_shell_layers", "1"),
            ("bottom_shell_layers", "1"),
            ("top_shell_thickness", "0"),
            ("bottom_shell_thickness", "0"),
        ]);
    }
    for &(key, value) in &settings {
        process.insert(key.into(), json!(value));
    }
    let mut filaments = selection
        .filaments
        .iter()
        .map(|name| profiles.filament(name).unwrap())
        .collect::<Vec<_>>();
    parity::normalize_filament_defaults(&mut filaments);
    let directory = tempfile::tempdir().unwrap();
    let model = directory.path().join("wide-slab.stl");
    std::fs::write(&model, slab(if shift.is_some() { 2.4 } else { 0.4 })).unwrap();
    let exported = runner
        .export_case(
            &CaseInputs {
                label,
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
    if shift.is_some() {
        let mut sparse = false;
        let mut relative_extrusion = false;
        let mut counted_layer = false;
        let mut sparse_layers = 0;
        for line in std::str::from_utf8(&sliced.reference).unwrap().lines() {
            if line.split_whitespace().next() == Some("M83") {
                relative_extrusion = true;
            } else if line.split_whitespace().next() == Some("M82") {
                relative_extrusion = false;
            } else if line == ";LAYER_CHANGE" {
                counted_layer = false;
            } else if let Some(role) = line.strip_prefix(";TYPE:") {
                sparse = role == "Sparse infill";
            } else if sparse
                && !counted_layer
                && (line.starts_with("G1 X") || line.starts_with("G1 Y"))
                && line
                    .split_whitespace()
                    .find_map(|word| word.strip_prefix('E'))
                    .is_some_and(|value| value.parse::<f64>().unwrap() > 0.0)
            {
                assert!(relative_extrusion, "fixture must use relative extrusion");
                sparse_layers += 1;
                counted_layer = true;
            }
        }
        assert!(
            sparse_layers >= 4,
            "expected multiple signed-shift layers, got {sparse_layers}"
        );
    }
    assert!(
        std::str::from_utf8(&sliced.reference)
            .unwrap()
            .contains(";TYPE:Bottom surface\n")
    );
    let outcome = parity::compare_case(&sliced);
    assert_eq!(outcome.status, "PASS", "{}", outcome.detail);
}

fn slab(height: f64) -> String {
    use std::fmt::Write;

    let points = [
        [0.0, 0.0, 0.0],
        [73.07148, 0.0, 0.0],
        [73.07148, 67.07148, 0.0],
        [0.0, 67.07148, 0.0],
        [0.0, 0.0, height],
        [73.07148, 0.0, height],
        [73.07148, 67.07148, height],
        [0.0, 67.07148, height],
    ];
    let mut stl = String::from("solid slab\n");
    for indices in [
        [0, 2, 1],
        [0, 3, 2],
        [4, 5, 6],
        [4, 6, 7],
        [0, 1, 5],
        [0, 5, 4],
        [1, 2, 6],
        [1, 6, 5],
        [2, 3, 7],
        [2, 7, 6],
        [3, 0, 4],
        [3, 4, 7],
    ] {
        stl.push_str("facet normal 0 0 0\nouter loop\n");
        for index in indices {
            let [x, y, z] = points[index];
            writeln!(stl, "vertex {x} {y} {z}").unwrap();
        }
        stl.push_str("endloop\nendfacet\n");
    }
    stl.push_str("endsolid slab\n");
    stl
}
