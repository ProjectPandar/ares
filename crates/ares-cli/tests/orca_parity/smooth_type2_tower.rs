//! `Print::_make_wipe_tower` / `WipeTower2` at the public project/output seam.

use crate::{CaseInputs, OrcaRunner, VendorProfiles, runner};
use serde_json::json;

#[path = "smooth_type2_tower/tools.rs"]
mod tools;

#[test]
fn orca_parity_smooth_type2_tower_complete_output() {
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
    for (mode, raft) in [("0", "0"), ("1", "0"), ("1", "2")] {
        process.insert("timelapse_type".into(), json!(mode));
        process.insert("raft_layers".into(), json!(raft));
        process.insert("raft_contact_distance".into(), json!("0.1"));
        let exported = runner
            .export_case(
                &CaseInputs {
                    label: &format!("smooth-type2/mode-{mode}-raft-{raft}"),
                    machine: &machine,
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
                "mode-{mode}-raft-{raft}: {} {}",
                outcome.status, outcome.detail
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
