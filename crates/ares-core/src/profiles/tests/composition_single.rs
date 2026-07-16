use crate::{ProjectSettings, compose_profile_fragments};

use super::{assert_invalid, fragments, selection};

#[test]
fn single_filament_composition_returns_concrete_project_settings() {
    let fragments = fragments([
        br#"{"type":"machine","name":"printer","printer_model":"Ares M"}"# as &[u8],
        br#"{"type":"process","name":"fine","layer_height":0.12,"compatible_printers":["printer",""]}"#,
        br#"{"type":"filament","name":"pla","filament_id":"PLA-ID","filament_diameter":[1.73],"filament_extruder_variant":["Direct Drive Standard","Direct Drive High Flow"]}"#,
    ]);
    let selection = selection("fine", "printer", ["pla"]);

    let composed = compose_profile_fragments(&fragments, &selection).unwrap();
    let settings: &ProjectSettings = composed.settings();

    assert_eq!(composed.process_name(), "fine");
    assert_eq!(composed.machine_name(), "printer");
    assert_eq!(composed.filament_names(), ["pla"]);
    assert_eq!(settings.printer.remaining.printer_model.0, "Ares M");
    assert_eq!(settings.process.object.layer_height.0, 0.12);
    assert_eq!(settings.filament.gcode.filament_diameter.0[0].0, 1.73);
    assert_eq!(settings.project.preset.print_settings_id.0, "fine");
    assert_eq!(settings.project.preset.printer_settings_id.0, "printer");
    assert_eq!(settings.project.preset.filament_settings_id.0, ["pla"]);
    assert_eq!(
        settings
            .project
            .gcode
            .filament_map
            .0
            .iter()
            .map(|value| value.0)
            .collect::<Vec<_>>(),
        [1]
    );
    assert_eq!(settings.project.gcode.filament_ids.0, ["PLA-ID"]);
    assert_eq!(
        settings
            .project
            .preset
            .filament_self_index
            .0
            .iter()
            .map(|value| value.0)
            .collect::<Vec<_>>(),
        [1, 1]
    );
    assert_eq!(
        settings.project.preset.print_compatible_printers.0,
        ["printer", ""]
    );
    assert_eq!(settings.metadata, Default::default());
}

#[test]
fn into_settings_preserves_the_typed_result_and_selected_names() {
    let fragments = fragments([
        br#"{"type":"machine","name":"m","printer_model":"M"}"# as &[u8],
        br#"{"type":"process","name":"p","layer_height":0.18}"#,
        br#"{"type":"filament","name":"f","filament_type":["PETG"]}"#,
    ]);
    let selection = selection("p", "m", ["f"]);
    let composed = compose_profile_fragments(&fragments, &selection).unwrap();

    assert_eq!(composed.process_name(), "p");
    assert_eq!(composed.machine_name(), "m");
    assert_eq!(composed.filament_names(), ["f"]);
    let settings: ProjectSettings = composed.into_settings();
    assert_eq!(settings.process.object.layer_height.0, 0.18);
    assert_eq!(settings.filament.gcode.filament_type.0, ["PETG"]);
}

#[test]
fn each_missing_selected_profile_is_an_atomic_invalid_input() {
    let fragments = fragments([
        br#"{"type":"machine","name":"m"}"# as &[u8],
        br#"{"type":"process","name":"p"}"#,
        br#"{"type":"filament","name":"f"}"#,
    ]);

    for selection in [
        selection("missing", "m", ["f"]),
        selection("p", "missing", ["f"]),
        selection("p", "m", ["missing"]),
    ] {
        let frozen_fragments = fragments.clone();
        let frozen_selection = selection.clone();
        assert_invalid(compose_profile_fragments(&fragments, &selection), "profile");
        assert_eq!(fragments, frozen_fragments);
        assert_eq!(selection, frozen_selection);
    }
}

#[test]
fn inherited_single_filament_is_resolved_before_composition() {
    let fragments = fragments([
        br#"{"type":"machine","name":"m"}"# as &[u8],
        br#"{"type":"process","name":"p"}"#,
        br#"{"type":"filament","name":"base","filament_diameter":[2.85],"filament_type":["PETG"]}"#,
        br#"{"type":"filament","name":"child","inherits":"base","filament_type":["PCTG"]}"#,
    ]);
    let selection = selection("p", "m", ["child"]);

    let settings = compose_profile_fragments(&fragments, &selection)
        .unwrap()
        .into_settings();
    assert_eq!(settings.filament.gcode.filament_diameter.0[0].0, 2.85);
    assert_eq!(settings.filament.gcode.filament_type.0, ["PCTG"]);
}

#[test]
fn inherited_and_cleared_process_compatibility_reaches_final_settings() {
    let fragments = fragments([
        br#"{"type":"machine","name":"m"}"# as &[u8],
        br#"{"type":"process","name":"base","compatible_printers":["M1",""],"compatible_printers_condition":"base-condition"}"#,
        br#"{"type":"process","name":"inherited","inherits":"base"}"#,
        br#"{"type":"process","name":"cleared","inherits":"base","compatible_printers":[],"compatible_printers_condition":""}"#,
        br#"{"type":"filament","name":"f"}"#,
    ]);

    let inherited =
        compose_profile_fragments(&fragments, &selection("inherited", "m", ["f"])).unwrap();
    assert_eq!(
        inherited
            .settings()
            .project
            .preset
            .print_compatible_printers
            .0,
        ["M1", ""]
    );
    assert_eq!(
        inherited
            .metadata()
            .compatible_machine_expression_group()
            .unwrap(),
        ["base-condition", ""]
    );

    let cleared = compose_profile_fragments(&fragments, &selection("cleared", "m", ["f"])).unwrap();
    assert!(
        cleared
            .settings()
            .project
            .preset
            .print_compatible_printers
            .0
            .is_empty()
    );
    assert_eq!(
        cleared.metadata().compatible_machine_expression_group(),
        None
    );
}

#[test]
fn inherited_filament_identity_and_variant_cardinality_reach_final_settings() {
    let fragments = fragments([
        br#"{"type":"machine","name":"m"}"# as &[u8],
        br#"{"type":"process","name":"p"}"#,
        br#"{"type":"filament","name":"base-a","filament_id":"A-ID","filament_extruder_variant":["A standard","A high flow"]}"#,
        br#"{"type":"filament","name":"child-a","inherits":"base-a","filament_id":"CHILD-A-ID"}"#,
        br#"{"type":"filament","name":"base-b","filament_id":"B-ID","filament_extruder_variant":["B standard","B high flow","B third"]}"#,
        br#"{"type":"filament","name":"child-b","inherits":"base-b","filament_id":"CHILD-B-ID"}"#,
    ]);

    let settings =
        compose_profile_fragments(&fragments, &selection("p", "m", ["child-a", "child-b"]))
            .unwrap()
            .into_settings();

    assert_eq!(settings.project.gcode.filament_ids.0, ["A-ID", "B-ID"]);
    assert_eq!(
        settings
            .project
            .preset
            .filament_self_index
            .0
            .iter()
            .map(|value| value.0)
            .collect::<Vec<_>>(),
        [1, 1, 2, 2, 2]
    );
    assert_eq!(settings.filament.gcode.filament_extruder_variant.0.len(), 5);
}
