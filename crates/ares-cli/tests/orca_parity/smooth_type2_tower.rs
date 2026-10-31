//! `Print::_make_wipe_tower` / `WipeTower2` at the public project/output seam.

use crate::{CaseInputs, OrcaRunner, VendorProfiles, runner};
use serde_json::json;

#[path = "smooth_type2_tower/chamber.rs"]
mod chamber;
#[path = "smooth_type2_tower/conflict.rs"]
mod conflict;
#[path = "smooth_type2_tower/islands.rs"]
mod islands;
#[path = "smooth_type2_tower/tools.rs"]
mod tools;
#[path = "smooth_type2_tower/variants.rs"]
mod variants;

#[test]
fn smooth_type2_tower_complete_output() {
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
        ("wipe_tower_wall_type", json!("rib")),
        ("wipe_tower_fillet_wall", json!("1")),
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
    for (mode, raft, footprint) in [
        ("0", "0", false),
        ("1", "0", false),
        ("1", "2", false),
        ("1", "2", true),
    ] {
        process.insert("timelapse_type".into(), json!(mode));
        process.insert("raft_layers".into(), json!(raft));
        process.insert("raft_contact_distance".into(), json!("0.1"));
        let mut case_machine = machine.clone();
        if footprint {
            process.insert("skirt_height".into(), json!("0"));
            process.insert("skirt_loops".into(), json!("0"));
            let start = case_machine["machine_start_gcode"]
                .as_str()
                .unwrap()
                .to_owned()
                + "\n; RAFT_MIN {first_layer_print_min[0]},{first_layer_print_min[1]}"
                + "\n; RAFT_MAX {first_layer_print_max[0]},{first_layer_print_max[1]}"
                + "\n; RAFT_HULL {size(first_layer_print_convex_hull)}";
            case_machine.insert("machine_start_gcode".into(), json!(start));
        }
        let exported = runner
            .export_case(
                &CaseInputs {
                    label: &format!("smooth-type2/mode-{mode}-raft-{raft}-footprint-{footprint}"),
                    machine: &case_machine,
                    process: &process,
                    filaments: &filaments,
                },
                &crate::smoke_overrides(),
                &runner::repo_root().join("tests/parity/cube10.stl"),
            )
            .unwrap();
        for (key, value) in [
            ("timelapse_type", json!(mode)),
            ("raft_layers", json!(raft)),
            ("raft_contact_distance", json!("0.1")),
            ("enable_prime_tower", json!("1")),
            ("wipe_tower_type", json!("type2")),
            ("wipe_tower_wall_type", json!("rib")),
            ("wipe_tower_fillet_wall", json!("1")),
            ("wipe_tower_x", json!(["15"])),
            ("wipe_tower_y", json!(["150"])),
        ] {
            runner::application::verify_value(&exported.project, key, &value).unwrap();
        }
        if footprint {
            for key in ["skirt_height", "skirt_loops"] {
                runner::application::verify_value(&exported.project, key, &json!("0")).unwrap();
            }
        }
        let sliced = runner.slice_case(&exported).unwrap();
        let reference = std::str::from_utf8(&sliced.reference).unwrap();
        assert_eq!(reference.contains(";TYPE:Prime tower\n"), mode == "1");
        if raft == "2" {
            assert!(
                reference.contains(";Z:0.7\n"),
                "reference must exercise the raft-gap tower layer"
            );
        }
        let outcome = crate::compare_case(&sliced);
        if outcome.status != "PASS" {
            failures.push(format!(
                "mode-{mode}-raft-{raft}-footprint-{footprint}: {} {}",
                outcome.status, outcome.detail
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
