use crate::compose_profile_fragments;

use super::{fragments, selection};

#[test]
fn positional_groups_preserve_missing_and_explicit_empty_interior_slots() {
    let fragments = fragments([
        br#"{"type":"machine","name":"machine-base"}"# as &[u8],
        br#"{"type":"machine","name":"m","inherits":"machine-base"}"#,
        br#"{"type":"process","name":"process-base"}"#,
        br#"{"type":"process","name":"p","inherits":"process-base","compatible_printers":["","m"],"compatible_printers_condition":"process-machine"}"#,
        br#"{"type":"filament","name":"missing-root"}"#,
        br#"{"type":"filament","name":"empty-root","inherits":"","compatible_printers_condition":"","compatible_prints_condition":""}"#,
        br#"{"type":"filament","name":"filament-base","compatible_printers_condition":"filament-machine","compatible_prints_condition":"filament-process"}"#,
        br#"{"type":"filament","name":"child","inherits":"filament-base"}"#,
    ]);
    let composed = compose_profile_fragments(
        &fragments,
        &selection("p", "m", ["missing-root", "empty-root", "child"]),
    )
    .unwrap();
    let metadata = composed.metadata();

    assert_eq!(
        metadata.inherits_group().unwrap(),
        ["process-base", "", "", "filament-base", "machine-base"]
    );
    assert_eq!(
        metadata.compatible_machine_expression_group().unwrap(),
        ["process-machine", "", "", "filament-machine"]
    );
    assert_eq!(
        metadata.compatible_process_expression_group().unwrap(),
        ["", "", "filament-process"]
    );
    assert_eq!(
        composed
            .settings()
            .project
            .preset
            .print_compatible_printers
            .0,
        ["", "m"]
    );
}

#[test]
fn all_empty_groups_are_absent_but_filament_id_slots_are_not_compacted() {
    let fragments = fragments([
        br#"{"type":"machine","name":"m"}"# as &[u8],
        br#"{"type":"process","name":"p","compatible_printers":[]}"#,
        br#"{"type":"filament","name":"a"}"#,
        br#"{"type":"filament","name":"b","inherits":""}"#,
    ]);
    let composed = compose_profile_fragments(&fragments, &selection("p", "m", ["a", "b"])).unwrap();
    let metadata = composed.metadata();

    assert_eq!(metadata.inherits_group(), None);
    assert_eq!(metadata.compatible_machine_expression_group(), None);
    assert_eq!(metadata.compatible_process_expression_group(), None);
    assert!(
        composed
            .settings()
            .project
            .preset
            .print_compatible_printers
            .0
            .is_empty()
    );
    assert_eq!(composed.settings().project.gcode.filament_ids.0, ["", ""]);
}
