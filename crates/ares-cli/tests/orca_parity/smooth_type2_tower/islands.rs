//! `Print::first_layer_islands`: normal spacing coverage versus tree lslices.
use crate::{CaseInputs, OrcaRunner, VendorProfiles, runner};
use serde_json::json;

#[test]
fn smooth_type2_tower_support_islands() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let profiles = VendorProfiles::load(
        &runner::repo_root().join("OrcaSlicer/resources/profiles"),
        "Creality",
    )
    .unwrap();
    let selection =
        crate::select_printer(&profiles, "Creality", "Creality Ender-3 0.4 nozzle").unwrap();
    let mut machine = profiles.machine(&selection.printer).unwrap();
    machine.insert("wipe_tower_type".into(), json!("type2"));
    let start = machine["machine_start_gcode"].as_str().unwrap().to_owned()
        + "\n; ISLAND_MIN {first_layer_print_min[0]},{first_layer_print_min[1]}"
        + "\n; ISLAND_MAX {first_layer_print_max[0]},{first_layer_print_max[1]}"
        + "\n; ISLAND_HULL {size(first_layer_print_convex_hull)}"
        + "\n; ISLAND_CENTER {first_layer_center_no_wipe_tower[0]},{first_layer_center_no_wipe_tower[1]}";
    machine.insert("machine_start_gcode".into(), json!(start));
    let mut filaments = selection
        .filaments
        .iter()
        .map(|name| profiles.filament(name).unwrap())
        .collect::<Vec<_>>();
    crate::normalize_filament_defaults(&mut filaments);
    for (kind, style) in [("normal(auto)", "grid"), ("tree(auto)", "tree_slim")] {
        let mut process = profiles.process(&selection.process).unwrap();
        crate::normalize_process_defaults(&machine, &mut process);
        let settings = [
            ("enable_prime_tower", "1"),
            ("timelapse_type", "1"),
            ("enable_support", "1"),
            ("support_type", kind),
            ("support_style", style),
            ("independent_support_layer_height", "1"),
            ("support_top_z_distance", "0.15"),
            ("support_bottom_z_distance", "0.2"),
            ("support_interface_top_layers", "3"),
            ("support_interface_bottom_layers", "-1"),
            ("skirt_height", "0"),
            ("skirt_loops", "0"),
        ];
        for (key, value) in settings {
            process.insert(key.into(), json!(value));
        }
        process.insert("wipe_tower_x".into(), json!(["15"]));
        process.insert("wipe_tower_y".into(), json!(["150"]));
        let exported = runner
            .export_case(
                &CaseInputs {
                    label: &format!("type2-support-islands/{kind}"),
                    machine: &machine,
                    process: &process,
                    filaments: &filaments,
                },
                &crate::smoke_overrides(),
                &runner::repo_root().join("tests/parity/overhang.stl"),
            )
            .unwrap();
        for (key, value) in settings {
            runner::application::verify_value(&exported.project, key, &json!(value)).unwrap();
        }
        let sliced = runner.slice_case(&exported).unwrap();
        let reference = std::str::from_utf8(&sliced.reference).unwrap();
        assert!(crate::option_coverage::normal_support_output::has_support_extrusion(reference));
        assert!(reference.contains(";TYPE:Prime tower\n"));
        let outcome = crate::compare_case(&sliced);
        assert_eq!(outcome.status, "PASS", "{kind}: {}", outcome.detail);
    }
}
