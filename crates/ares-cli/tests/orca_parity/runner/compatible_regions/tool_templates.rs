use serde_json::json;

#[path = "tool_templates/fixture.rs"]
mod fixture;
use fixture::outputs;

#[path = "tool_templates/change.rs"]
mod change;

#[path = "tool_templates/timing.rs"]
mod timing;

#[path = "tool_templates/auto_lift.rs"]
mod auto_lift;

#[path = "tool_templates/detour.rs"]
mod detour;

#[path = "tool_templates/seam.rs"]
mod seam;

#[path = "tool_templates/filament_stats.rs"]
mod filament_stats;

#[path = "tool_templates/cooling.rs"]
mod cooling;

#[path = "tool_templates/layer_context.rs"]
mod layer_context;

#[path = "tool_templates/timelapse_context.rs"]
mod timelapse_context;

#[path = "tool_templates/pre_heating.rs"]
mod pre_heating;

#[path = "tool_templates/printer_columns.rs"]
mod printer_columns;

#[test]
fn printing_toolchange_restores_pressure_advance_after_filament_start() {
    if !crate::runner::reference_enabled("ARES_PARITY_REGION_FILAMENT_REFERENCE") {
        return;
    }
    let (reference, actual) = outputs("; pressure start {filament_extruder_id}", "", false, &[]);
    for (label, bytes) in [("native", &reference), ("Ares", &actual)] {
        let output = std::str::from_utf8(bytes).unwrap();
        let body = &output
            [output.find(";LAYER_CHANGE").unwrap()..output.find("; EXECUTABLE_BLOCK_END").unwrap()];
        let mut transitions = 0;
        for (offset, _) in body.match_indices("; pressure start ") {
            let transition = &body[offset..];
            let object = transition.find("; printing object").unwrap();
            assert!(
                transition[..object]
                    .lines()
                    .any(|line| line == "M900 K0.02; Override pressure advance value"),
                "{label}: pressure advance missing after printing filament-start"
            );
            transitions += 1;
        }
        assert_eq!(
            transitions, 16,
            "{label}: every scheduled printing transition"
        );
    }
}

#[test]
fn first_printing_toolchange_renders_old_end_then_new_start() {
    if !crate::runner::reference_enabled("ARES_PARITY_REGION_FILAMENT_REFERENCE") {
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
    if !crate::runner::reference_enabled("ARES_PARITY_REGION_FILAMENT_REFERENCE") {
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
    if !crate::runner::reference_enabled("ARES_PARITY_REGION_FILAMENT_REFERENCE") {
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
    if !crate::runner::reference_enabled("ARES_PARITY_REGION_FILAMENT_REFERENCE") {
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
    if !crate::runner::reference_enabled("ARES_PARITY_REGION_FILAMENT_REFERENCE") {
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

#[test]
fn scheduled_toolchange_total_is_shared_by_templates_and_summary() {
    if !crate::runner::reference_enabled("ARES_PARITY_REGION_FILAMENT_REFERENCE") {
        return;
    }
    let (reference, actual) = outputs(
        "; scheduled start {total_toolchanges} {total_toolchanges / 3}",
        "; scheduled end {total_toolchanges} {total_toolchanges / 3}",
        false,
        &[],
    );
    for (label, bytes) in [("native", &reference), ("Ares", &actual)] {
        let output = std::str::from_utf8(bytes).unwrap();
        for prefix in ["; scheduled start ", "; scheduled end "] {
            let lines = output
                .lines()
                .filter(|line| line.starts_with(prefix))
                .collect::<Vec<_>>();
            assert!(
                !lines.is_empty(),
                "{label}: scheduled-count template absent"
            );
            for line in lines {
                assert_eq!(
                    line.strip_prefix(prefix),
                    Some("16 5"),
                    "{label}: immutable integer prediction"
                );
            }
        }
        assert!(
            output
                .lines()
                .any(|line| line == "; total filament change = 16"),
            "{label}: compatible summary shares the prediction"
        );
    }
}
