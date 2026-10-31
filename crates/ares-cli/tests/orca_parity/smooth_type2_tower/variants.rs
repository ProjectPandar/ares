//! `WipeTower2` shapes/sparse layers and `Print` occupancy at full-output seam.

use crate::{CaseInputs, OrcaRunner, VendorProfiles, runner};
use serde_json::json;

#[test]
fn smooth_type2_tower_shapes_and_sparse_layers() {
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
    let mut process = profiles.process(&selection.process).unwrap();
    crate::normalize_process_defaults(&machine, &mut process);
    for (key, value) in [
        ("enable_prime_tower", json!("1")),
        ("timelapse_type", json!("1")),
        ("wipe_tower_wall_type", json!("rib")),
        ("wipe_tower_fillet_wall", json!("1")),
        ("prime_tower_width", json!("40")),
        ("wipe_tower_x", json!(["15"])),
        ("wipe_tower_y", json!(["150"])),
    ] {
        process.insert(key.into(), value);
    }
    let mut filaments = selection
        .filaments
        .iter()
        .map(|name| profiles.filament(name).unwrap())
        .collect::<Vec<_>>();
    crate::normalize_filament_defaults(&mut filaments);
    let mut failures = Vec::new();
    for (label, changes) in [
        ("rectangle", json!({"wipe_tower_wall_type": "rectangle"})),
        ("cone", json!({"wipe_tower_wall_type": "cone"})),
        ("rib-sharp", json!({"wipe_tower_fillet_wall": "0"})),
        ("rib-auto-brim", json!({"prime_tower_brim_width": "-1"})),
        ("rib-thin", json!({"layer_height": "0.08"})),
        ("rib-arcs", json!({"enable_arc_fitting": "1"})),
        ("rib-no-sparse", json!({"wipe_tower_no_sparse_layers": "1"})),
        ("rib-no-skirt", json!({"skirt_loops": "0"})),
        ("bounds-no-skirt", json!({"skirt_loops": "0"})),
        (
            "bounds-no-skirt-height",
            json!({"skirt_loops": "0", "skirt_height": "0"}),
        ),
        ("bounds-with-skirt", json!({"skirt_loops": "1"})),
        (
            "brim-near-tower",
            json!({
                "brim_type": "outer_only", "brim_width": "8",
                "brim_object_gap": "0.1", "skirt_loops": "0",
                "prime_tower_width": "10", "wipe_tower_x": ["90"],
                "wipe_tower_y": ["105"]
            }),
        ),
        (
            "rib-rotated-offset",
            json!({
                "wipe_tower_rotation_angle": "37",
                "wipe_tower_x": ["60"],
                "wipe_tower_y": ["120"]
            }),
        ),
        (
            "singleton-priming",
            json!({"single_extruder_multi_material_priming": "1"}),
        ),
    ] {
        let hull_vertices = match label {
            "bounds-no-skirt" => Some(16),
            "bounds-no-skirt-height" => Some(6),
            "bounds-with-skirt" => Some(18),
            _ => None,
        };
        let mut case_process = process.clone();
        let changes = changes.as_object().unwrap();
        case_process.extend(changes.clone());
        let mut case_machine = machine.clone();
        if let Some(vertices) = hull_vertices {
            let start = case_machine["machine_start_gcode"]
                .as_str()
                .unwrap()
                .to_owned();
            use std::fmt::Write as _;
            let mut start = start
                + "\n; WT {has_wipe_tower} {has_single_extruder_multi_material_priming} {total_toolchanges}"
                + "\n; MIN {first_layer_print_min[0]},{first_layer_print_min[1]}"
                + "\n; MAX {first_layer_print_max[0]},{first_layer_print_max[1]}"
                + "\n; CENTER {first_layer_center_no_wipe_tower[0]},{first_layer_center_no_wipe_tower[1]}"
                + "\n; HULL_SIZE {size(first_layer_print_convex_hull)}";
            for index in 0..vertices {
                write!(
                    start,
                    "\n; HULL_{index} {{first_layer_print_convex_hull[{index}]}}"
                )
                .unwrap();
            }
            case_machine.insert("machine_start_gcode".into(), json!(start));
        }
        if label == "rib-rotated-offset" {
            case_machine.insert("extruder_offset".into(), json!(["1.25x-0.75"]));
        }
        if label == "singleton-priming" {
            case_machine.insert("single_extruder_multi_material".into(), json!("1"));
        }
        let exported = runner
            .export_case(
                &CaseInputs {
                    label: &format!("smooth-type2/{label}"),
                    machine: &case_machine,
                    process: &case_process,
                    filaments: &filaments,
                },
                &crate::smoke_overrides(),
                &runner::repo_root().join("tests/parity/cube10.stl"),
            )
            .unwrap();
        for (key, value) in changes {
            runner::application::verify_value(&exported.project, key, value).unwrap();
        }
        for key in [
            "enable_prime_tower",
            "timelapse_type",
            "wipe_tower_wall_type",
        ] {
            runner::application::verify_value(&exported.project, key, &case_process[key]).unwrap();
        }
        if label == "rib-rotated-offset" {
            runner::application::verify_value(
                &exported.project,
                "extruder_offset",
                &case_machine["extruder_offset"],
            )
            .unwrap();
        }
        if label == "singleton-priming" {
            runner::application::verify_value(
                &exported.project,
                "single_extruder_multi_material",
                &json!("1"),
            )
            .unwrap();
        }
        let sliced = runner.slice_case(&exported).unwrap();
        let reference = std::str::from_utf8(&sliced.reference).unwrap();
        assert!(reference.contains(";TYPE:Prime tower\n"), "{label}");
        if let Some(vertices) = hull_vertices {
            assert!(
                reference.contains(&format!("; HULL_SIZE {vertices}\n")),
                "{label}"
            );
        }
        if label == "singleton-priming" {
            assert!(reference.contains("; CP PRIMING START\n"));
        }
        let outcome = crate::compare_case(&sliced);
        if outcome.status != "PASS" {
            failures.push(format!("{label}: {} {}", outcome.status, outcome.detail));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
