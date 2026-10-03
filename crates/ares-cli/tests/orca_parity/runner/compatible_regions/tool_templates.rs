use super::super::{CaseInputs, OrcaRunner, application, repo_root};
use crate::{self as parity, presets::VendorProfiles};
use serde_json::json;

#[test]
fn first_printing_toolchange_renders_old_end_then_new_start() {
    if std::env::var("ARES_PARITY_REGION_FILAMENT_REFERENCE").as_deref() != Ok("1") {
        return;
    }
    let (reference, actual) = outputs(
        "; parity filament start {filament_extruder_id}",
        "; parity filament end {filament_extruder_id}",
        false,
        &[],
    );
    for (label, bytes) in [("native", &reference), ("Ares", &actual)] {
        let output = std::str::from_utf8(bytes).unwrap();
        let body = &output[output.find(";LAYER_CHANGE").unwrap()..];
        let tool = body.find("\nT1\n").unwrap();
        assert!(
            body[..tool]
                .lines()
                .any(|line| line == "; parity filament end 0"),
            "{label}: old-tool end missing before first printing T1"
        );
        let start = body[tool..].find("; parity filament start 1\n").unwrap();
        let object = body[tool..].find("; printing object").unwrap();
        assert!(
            start < object,
            "{label}: new-tool start must precede its object body"
        );
    }
}

#[test]
fn printing_tool_templates_use_selected_cut_and_ec_aliases() {
    if std::env::var("ARES_PARITY_REGION_FILAMENT_REFERENCE").as_deref() != Ok("1") {
        return;
    }
    let (reference, actual) = outputs(
        "; alias start {filament_extruder_id} {retraction_distance_when_cut} {long_retraction_when_cut} {retraction_distance_when_ec} {long_retraction_when_ec}",
        "; alias end {filament_extruder_id} {retraction_distance_when_cut} {long_retraction_when_cut} {retraction_distance_when_ec} {long_retraction_when_ec}",
        true,
        &[],
    );
    for (label, bytes) in [("native", &reference), ("Ares", &actual)] {
        let output = std::str::from_utf8(bytes).unwrap();
        let body = &output[output.find(";LAYER_CHANGE").unwrap()..];
        let tool = body.find("\nT1\n").unwrap();
        assert!(
            body[..tool]
                .lines()
                .any(|line| line == "; alias end 0 10 false 3 false"),
            "{label}: old-tool aliases"
        );
        let next_object = body[tool..].find("; printing object").unwrap() + tool;
        assert!(
            body[tool..next_object]
                .lines()
                .any(|line| line == "; alias start 1 18 true 7 true"),
            "{label}: new-tool aliases"
        );
        let next_tool = body[next_object..].find("\nT0\n").unwrap() + next_object;
        assert!(
            body[next_object..next_tool]
                .lines()
                .any(|line| line == "; alias end 1 18 true 7 true"),
            "{label}: previously selected tool aliases"
        );
    }
}

#[test]
fn legacy_vectors_use_logical_tool_context_and_preserve_explicit_indices() {
    if std::env::var("ARES_PARITY_REGION_FILAMENT_REFERENCE").as_deref() != Ok("1") {
        return;
    }
    let (reference, actual) = outputs(
        "; legacy start {filament_extruder_id} [retraction_distances_when_ec] {retraction_distances_when_ec[0]} {retraction_distances_when_ec[1]}",
        "; legacy end {filament_extruder_id} [retraction_distances_when_ec] {retraction_distances_when_ec[0]} {retraction_distances_when_ec[1]}",
        true,
        &[],
    );
    for (label, bytes) in [("native", &reference), ("Ares", &actual)] {
        let output = std::str::from_utf8(bytes).unwrap();
        let body = &output[output.find(";LAYER_CHANGE").unwrap()..];
        let tool = body.find("\nT1\n").unwrap();
        assert!(
            body[..tool]
                .lines()
                .any(|line| line == "; legacy end 0 3 3 7"),
            "{label}: old-tool legacy context"
        );
        let next_object = body[tool..].find("; printing object").unwrap() + tool;
        assert!(
            body[tool..next_object]
                .lines()
                .any(|line| line == "; legacy start 1 7 3 7"),
            "{label}: new-tool legacy context and explicit indices"
        );
        let next_tool = body[next_object..].find("\nT0\n").unwrap() + next_object;
        assert!(
            body[next_object..next_tool]
                .lines()
                .any(|line| line == "; legacy end 1 7 3 7"),
            "{label}: later old-tool legacy context"
        );
    }
}

#[test]
fn printing_toolchange_retracts_old_nozzle_before_filament_end() {
    if std::env::var("ARES_PARITY_REGION_FILAMENT_REFERENCE").as_deref() != Ok("1") {
        return;
    }
    let (reference, actual) = outputs(
        "; retract start {filament_extruder_id}",
        "; retract end {filament_extruder_id} {e_retracted[current_extruder]} {e_restart_extra[current_extruder]}",
        false,
        &[
            ("retract_length_toolchange", json!(["6", "9"])),
            ("retract_restart_extra_toolchange", json!(["0.25", "0.5"])),
        ],
    );
    // Snapmaker's current printer variant expands its first value to both
    // physical nozzles at slicing time, despite preserving both in the 3MF.
    let reference_config = std::str::from_utf8(&reference).unwrap();
    assert!(reference_config.contains("; retract_length_toolchange = 6,6\n"));
    assert!(reference_config.contains("; retract_restart_extra_toolchange = 0.25,0.25\n"));
    for (label, bytes) in [("native", &reference), ("Ares", &actual)] {
        let output = std::str::from_utf8(bytes).unwrap();
        let body = &output[output.find(";LAYER_CHANGE").unwrap()..];
        let first_end = body.find("; retract end ").unwrap();
        assert_eq!(
            body[first_end..].lines().next().unwrap(),
            "; retract end 0 6 0.25",
            "{label}: toolchange retract and restart extra precede the old end"
        );
        let second_end = body[first_end + 1..].find("; retract end ").unwrap() + first_end + 1;
        assert_eq!(
            body[second_end..].lines().next().unwrap(),
            "; retract end 1 6 0.25",
            "{label}: the later nozzle also receives the normalized toolchange settings"
        );
    }
}

#[test]
fn filament_header_lists_scheduled_logical_ids_not_their_count() {
    if std::env::var("ARES_PARITY_REGION_FILAMENT_REFERENCE").as_deref() != Ok("1") {
        return;
    }
    let (reference, actual) = outputs("", "", false, &[]);
    for (label, bytes) in [("native", &reference), ("Ares", &actual)] {
        let output = std::str::from_utf8(bytes).unwrap();
        assert_eq!(
            output.lines().find(|line| line.starts_with("; filament: ")),
            Some("; filament: 1,2"),
            "{label}: header identifies both scheduled logical filaments"
        );
    }
}

fn outputs(
    start: &str,
    end: &str,
    aliases: bool,
    machine_overrides: &[(&str, serde_json::Value)],
) -> (Vec<u8>, Vec<u8>) {
    let runner = OrcaRunner::from_env().unwrap().unwrap();
    let profiles = VendorProfiles::load(
        &repo_root().join("OrcaSlicer/resources/profiles"),
        "Snapmaker",
    )
    .unwrap();
    let selection =
        parity::select_printer(&profiles, "Snapmaker", "Snapmaker J1 (0.4 nozzle)").unwrap();
    let mut machine = profiles.machine(&selection.printer).unwrap();
    machine.insert("filament_map_mode".into(), json!("Manual"));
    machine.insert("filament_map".into(), json!(["1", "2"]));
    for (key, value) in machine_overrides {
        machine.insert((*key).into(), value.clone());
    }
    if aliases {
        machine.insert("enable_long_retraction_when_cut".into(), json!("2"));
        machine.insert("retraction_distances_when_cut".into(), json!(["10", "18"]));
        machine.insert("long_retractions_when_cut".into(), json!(["0", "1"]));
    }
    let mut process = profiles.process(&selection.process).unwrap();
    parity::normalize_process_defaults(&machine, &mut process);
    for (key, value) in [
        ("enable_support", "0"),
        ("enable_prime_tower", "0"),
        ("sparse_infill_density", "0%"),
        ("sparse_infill_pattern", "rectilinear"),
        ("sparse_infill_filament_id", "2"),
    ] {
        process.insert(key.into(), json!(value));
    }
    let mut filaments = vec![profiles.filament(&selection.filaments[0]).unwrap(); 2];
    parity::normalize_filament_defaults(&mut filaments);
    for (filament, colour) in filaments.iter_mut().zip(["#F2754E", "#0000FF"]) {
        filament.insert("filament_colour".into(), json!([colour]));
        filament.insert("filament_start_gcode".into(), json!([start]));
        filament.insert("filament_end_gcode".into(), json!([end]));
    }
    if aliases {
        for (filament, (distance, enabled)) in filaments.iter_mut().zip([("3", "0"), ("7", "1")]) {
            filament.insert("retraction_distances_when_ec".into(), json!([distance]));
            filament.insert("long_retractions_when_ec".into(), json!([enabled]));
        }
        for (filament, (distance, enabled)) in filaments.iter_mut().zip([("10", "0"), ("18", "1")])
        {
            filament.insert(
                "filament_retraction_distances_when_cut".into(),
                json!([distance]),
            );
            filament.insert(
                "filament_long_retractions_when_cut".into(),
                json!([enabled]),
            );
        }
    }
    let exported = runner
        .export_case(
            &CaseInputs {
                label: "compatible-regions/printing-tool-template-lifecycle",
                machine: &machine,
                process: &process,
                filaments: &filaments,
            },
            &parity::smoke_overrides(),
            &repo_root().join("tests/parity/overhang.stl"),
        )
        .unwrap();
    let exported = runner
        .bind_filament_map_mode(exported, &json!("Manual"))
        .unwrap();
    application::verify_value(
        &exported.project,
        "filament_start_gcode",
        &json!([start, start]),
    )
    .unwrap();
    application::verify_value(&exported.project, "filament_end_gcode", &json!([end, end])).unwrap();
    for (key, value) in machine_overrides {
        application::verify_value(&exported.project, key, value).unwrap();
    }
    if aliases {
        for (key, expected) in [
            ("enable_long_retraction_when_cut", json!("2")),
            ("retraction_distances_when_cut", json!(["10", "18"])),
            ("long_retractions_when_cut", json!(["0", "1"])),
            (
                "filament_retraction_distances_when_cut",
                json!(["10", "18"]),
            ),
            ("filament_long_retractions_when_cut", json!(["0", "1"])),
            ("retraction_distances_when_ec", json!(["3", "7"])),
            ("long_retractions_when_ec", json!(["0", "1"])),
        ] {
            application::verify_value(&exported.project, key, &expected).unwrap();
        }
    }
    let sliced = runner.slice_case(&exported).unwrap();
    let complete = parity::compare_case(&sliced);
    eprintln!("complete output: {}: {}", complete.status, complete.detail);
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    let actual = runtime
        .block_on(ares_core::slice_project(
            &sliced.project,
            ares_core::GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
            ares_core::SlicerContext::new(0),
        ))
        .unwrap();
    (sliced.reference, actual)
}
