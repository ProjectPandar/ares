//! `GCode.cpp:3077–3113` consumes the generated tower mesh bottom area.
use crate::{CaseInputs, OrcaRunner, VendorProfiles, runner};
use serde_json::json;

#[test]
fn smooth_type2_tower_flat_chamber() {
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
        + "\n; FLAT_CHAMBER {hold_chamber_temp_for_flat_print}";
    machine.insert("machine_start_gcode".into(), json!(start));
    let end = machine["machine_end_gcode"].as_str().unwrap().to_owned()
        + "\n; END_FLAT_CHAMBER {hold_chamber_temp_for_flat_print}";
    machine.insert("machine_end_gcode".into(), json!(end));
    let mut process = profiles.process(&selection.process).unwrap();
    crate::normalize_process_defaults(&machine, &mut process);
    for (key, value) in [
        ("timelapse_type", json!("1")),
        ("wipe_tower_wall_type", json!("rectangle")),
        ("prime_tower_width", json!("80")),
        ("prime_tower_brim_width", json!("10")),
        ("wipe_tower_x", json!(["15"])),
        ("wipe_tower_y", json!(["150"])),
        ("brim_type", json!("no_brim")),
    ] {
        process.insert(key.into(), value);
    }
    let mut filaments = selection
        .filaments
        .iter()
        .map(|name| profiles.filament(name).unwrap())
        .collect::<Vec<_>>();
    crate::normalize_filament_defaults(&mut filaments);
    let directory = tempfile::tempdir().unwrap();
    let mesh = directory.path().join("flat-cube.stl");
    let mut bytes = std::fs::read(runner::repo_root().join("tests/parity/cube10.stl")).unwrap();
    for triangle in bytes[84..].as_chunks_mut::<50>().0 {
        for vertex in triangle[12..48].as_chunks_mut::<12>().0 {
            let z = f32::from_le_bytes(vertex[8..12].try_into().unwrap()) * 0.02;
            vertex[8..12].copy_from_slice(&z.to_le_bytes());
        }
    }
    std::fs::write(&mesh, bytes).unwrap();
    let mut failures = Vec::new();
    for (enabled, wall, expected) in [
        ("0", "rectangle", "false"),
        ("1", "rectangle", "true"),
        ("1", "rib", "true"),
    ] {
        process.insert("enable_prime_tower".into(), json!(enabled));
        process.insert("wipe_tower_wall_type".into(), json!(wall));
        let exported = runner
            .export_case(
                &CaseInputs {
                    label: &format!("smooth-type2/flat-chamber-{enabled}-{wall}"),
                    machine: &machine,
                    process: &process,
                    filaments: &filaments,
                },
                &crate::smoke_overrides(),
                &mesh,
            )
            .unwrap();
        for key in [
            "enable_prime_tower",
            "wipe_tower_wall_type",
            "prime_tower_width",
            "prime_tower_brim_width",
            "brim_type",
        ] {
            runner::application::verify_value(&exported.project, key, &process[key]).unwrap();
        }
        let sliced = runner.slice_case(&exported).unwrap();
        let reference = std::str::from_utf8(&sliced.reference).unwrap();
        assert!(
            reference.contains(&format!("; FLAT_CHAMBER {expected}\n")),
            "tower={enabled}"
        );
        assert!(reference.contains(&format!("; END_FLAT_CHAMBER {expected}\n")));
        let outcome = crate::compare_case(&sliced);
        if outcome.status != "PASS" {
            failures.push(format!(
                "tower={enabled} wall={wall}: {} {}",
                outcome.status, outcome.detail
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
