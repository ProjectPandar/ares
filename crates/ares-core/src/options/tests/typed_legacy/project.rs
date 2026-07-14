use crate::{
    ExtruderType, ProcessSupportBasePattern, ProcessSupportStyle, ProcessSupportType,
    ProcessWallSequence, ProcessWipeTowerWallType, ProjectSettings,
};

use super::super::project_fixture::project_settings_bytes;

#[test]
fn executable_action_families_reach_concrete_typed_targets() {
    let settings = parse(
        r#"{
            "initial_layer_flow_ratio":"0.85",
            "wall_filament":"1",
            "initial_layer_speed":"125%",
            "top_one_wall_type":"top",
            "prime_tower_rib_wall":"1",
            "support_base_pattern":"none",
            "wall_infill_order":"outer wall/inner wall/infill",
            "extruder_type":["DirectDrive","Bowden"],
            "filament_type":["ASA-Aero","PLA"]
        }"#,
    );

    assert_eq!(
        settings.process.region.bottom_solid_infill_flow_ratio.0,
        0.85
    );
    assert_eq!(settings.process.region.outer_wall_filament_id.0, 0);
    assert_eq!(settings.process.print.initial_layer_speed.0, 30.0);
    assert!(settings.process.region.only_one_wall_top.0);
    assert_eq!(
        settings.process.print.wipe_tower_wall_type,
        ProcessWipeTowerWallType::Rib
    );
    assert_eq!(
        settings.process.object.support_base_pattern,
        ProcessSupportBasePattern::Hollow
    );
    assert_eq!(
        settings.process.region.wall_sequence,
        ProcessWallSequence::OuterInner
    );
    assert_eq!(
        settings.printer.gcode.extruder_type.0,
        vec![ExtruderType::DirectDrive, ExtruderType::Bowden]
    );
    assert_eq!(
        settings.filament.gcode.filament_type.0,
        ["ASA-AERO", "PLA"]
    );
}

#[test]
fn legacy_array_rename_is_flattened_then_typed() {
    let settings = parse(r#"{"bridge_fan_speed":["80","95"]}"#);
    assert_eq!(
        settings
            .filament
            .print
            .overhang_fan_speed
            .0
            .iter()
            .map(|value| value.0)
            .collect::<Vec<_>>(),
        [80, 95]
    );
}

#[test]
fn bool_and_cstyle_string_arrays_decode_through_concrete_targets() {
    let settings = parse(
        r#"{
            "cooling":["1","0"],
            "printer_extruder_variant":["quote\"","slash\\","line\n","return\r","tab\t"]
        }"#,
    );
    assert_eq!(
        settings
            .filament
            .print
            .slow_down_for_layer_cooling
            .0
            .iter()
            .map(|value| value.0)
            .collect::<Vec<_>>(),
        [true, false]
    );
    assert_eq!(
        settings.printer.gcode.printer_extruder_variant.0,
        ["quote\"", "slash\\", "line\n", "return\r", "tab\t"]
    );
}

#[test]
fn scalar_filament_type_uses_fixed_cstyle_tokenizer() {
    let valid: &[(&str, &[&str])] = &[
        (r#""a\qb""#, &["aqb"]),
        (" \t\"a\"\t ; \t\"b\"\t", &["a", "b"]),
        ("\t  a", &["a"]),
        (r#"a"""#, &[r#"a"""#]),
        (r#"a#b"#, &["a#b"]),
        (r#"a\q"#, &[r#"a\q"#]),
        ("a;", &["a", ""]),
        ("a;;", &["a", "", ""]),
        ("a; ", &["a"]),
        ("a; ;", &["a", "", ""]),
    ];

    for &(lexical, expected) in valid {
        assert_eq!(
            filament_types_from_lexical(lexical).unwrap(),
            expected,
            "lexical bytes: {:?}",
            lexical.as_bytes()
        );
    }

    for lexical in [r#""a"x"#, r#""a""b""#, r##""a"#"b""##] {
        let error = filament_types_from_lexical(lexical).unwrap_err();
        assert!(error.contains("filament_type"), "{error}");
    }
}

#[test]
fn obsolete_assignment_is_the_only_silently_consumed_unknown_input() {
    for input in [
        r#"{"acceleration":"ignored"}"#,
        r#"{"acceleration":[1,true,null]}"#,
    ] {
        assert_eq!(parse(input), ProjectSettings::default());
    }
    for input in [
        r#"{"acceleration":{"invalid":"shape"}}"#,
        r#"{"acceleration":true}"#,
        r#"{"acceleration":7}"#,
    ] {
        assert_error_contains(input, &["acceleration"]);
    }
}

#[test]
fn legacy_sources_reject_native_json_scalars() {
    assert_error_contains(r#"{"initial_layer_speed":42}"#, &["initial_layer_speed"]);
}

#[test]
fn a_consumed_assignment_does_not_claim_canonical_target_presence() {
    let settings = parse(
        r#"{"initial_layer_speed":"125%","initial_layer_speed":"42"}"#,
    );
    assert_eq!(settings.process.print.initial_layer_speed.0, 42.0);
}

#[test]
fn prime_tower_rib_trigger_claims_its_concrete_target_presence() {
    for input in [
        r#"{"prime_tower_rib_wall":"1","wipe_tower_wall_type":"rectangle"}"#,
        r#"{"wipe_tower_wall_type":"rectangle","prime_tower_rib_wall":"1"}"#,
    ] {
        assert_error_contains(input, &["duplicate Orca option wipe_tower_wall_type"]);
    }
}

#[test]
fn canonical_and_legacy_spellings_share_strict_duplicate_presence() {
    for input in [
        r#"{"bottom_solid_infill_flow_ratio":"0.7","initial_layer_flow_ratio":"0.8"}"#,
        r#"{"initial_layer_flow_ratio":"0.8","bottom_solid_infill_flow_ratio":"0.7"}"#,
    ] {
        assert_error_contains(
            input,
            &["duplicate Orca option bottom_solid_infill_flow_ratio"],
        );
    }
}

#[test]
fn two_legacy_spellings_for_one_target_are_strict_duplicates() {
    for input in [
        r#"{"wall_filament":"2","perimeter_extruder":"3"}"#,
        r#"{"perimeter_extruder":"3","wall_filament":"2"}"#,
    ] {
        assert_error_contains(input, &["duplicate Orca option outer_wall_filament_id"]);
    }
}

#[test]
fn unknown_and_deferred_inputs_report_the_exact_source_name() {
    for source in [
        "future_option",
        "inherits_cummulative",
        "compatible_printers_condition_cummulative",
        "compatible_prints_condition_cummulative",
        "different_settings_to_system",
    ] {
        let input = format!(r#"{{"{source}":"value"}}"#);
        assert_error_contains(&input, &[source]);
    }

    let settings = ProjectSettings::default();
    for group in [
        serde_json::to_string(&settings.printer).unwrap(),
        serde_json::to_string(&settings.process).unwrap(),
        serde_json::to_string(&settings.filament).unwrap(),
        serde_json::to_string(&settings.project).unwrap(),
        serde_json::to_string(&settings.metadata).unwrap(),
    ] {
        assert!(!group.contains("different_settings_to_system"));
    }
}

#[test]
fn support_hybrid_derived_style_wins_in_both_orders() {
    for input in [
        r#"{"support_style":"grid","support_type":"hybrid(auto)"}"#,
        r#"{"support_type":"hybrid(auto)","support_style":"grid"}"#,
    ] {
        let settings = parse(input);
        assert_eq!(
            settings.process.object.support_type,
            ProcessSupportType::TreeAuto
        );
        assert_eq!(
            settings.process.object.support_style,
            ProcessSupportStyle::TreeHybrid
        );
    }
}

#[test]
fn support_nontrigger_preserves_explicit_style() {
    let settings = parse(r#"{"support_type":"tree(auto)","support_style":"grid"}"#);
    assert_eq!(
        settings.process.object.support_style,
        ProcessSupportStyle::Grid
    );
}

#[test]
fn both_infill_first_spellings_override_explicit_false_in_both_orders() {
    for spelling in [
        "infill/outer wall/inner wall",
        "infill/inner wall/outer wall",
    ] {
        for input in [
            format!(r#"{{"is_infill_first":"0","wall_infill_order":"{spelling}"}}"#),
            format!(r#"{{"wall_infill_order":"{spelling}","is_infill_first":"0"}}"#),
        ] {
            let settings = parse(&input);
            assert!(settings.process.region.is_infill_first.0);
        }
    }
}

#[test]
fn wall_order_nontrigger_preserves_explicit_infill_first() {
    let settings = parse(
        r#"{"wall_infill_order":"inner wall/outer wall/infill","is_infill_first":"1"}"#,
    );
    assert!(settings.process.region.is_infill_first.0);
}

#[test]
fn invalid_legacy_array_token_names_source_and_concrete_target() {
    assert_error_contains(
        r#"{"bridge_fan_speed":["80","not-an-int"]}"#,
        &["bridge_fan_speed", "overhang_fan_speed"],
    );
}

#[test]
fn nested_flattening_is_rejected_by_the_flat_concrete_vector_target() {
    for input in [
        r#"{"printer_extruder_variant":[["a"],["b"]]}"#,
        r#"{"printer_extruder_variant":[[],["b"]]}"#,
        r#"{"printer_extruder_variant":[[],[]]}"#,
    ] {
        assert_error_contains(input, &["printer_extruder_variant"]);
    }
}

#[test]
fn array_element_group_separator_is_a_literal_string_byte() {
    let settings = parse(r#"{"printer_extruder_variant":["a#b"]}"#);
    assert_eq!(
        settings.printer.gcode.printer_extruder_variant.0,
        ["a#b"]
    );
}

#[test]
fn real_canonical_project_settings_still_load_strictly() {
    serde_json::from_slice::<ProjectSettings>(&project_settings_bytes()).unwrap();
}

fn parse(input: &str) -> ProjectSettings {
    serde_json::from_str(input).unwrap()
}

fn filament_types_from_lexical(lexical: &str) -> Result<Vec<String>, String> {
    let encoded = serde_json::to_string(lexical).unwrap();
    let decoded: String = serde_json::from_str(&encoded).unwrap();
    assert_eq!(decoded.as_bytes(), lexical.as_bytes());
    let input = format!(r#"{{"filament_type":{encoded}}}"#);
    serde_json::from_str::<ProjectSettings>(&input)
        .map(|settings| settings.filament.gcode.filament_type.0)
        .map_err(|error| error.to_string())
}

fn assert_error_contains(input: &str, expected: &[&str]) {
    let error = serde_json::from_str::<ProjectSettings>(input)
        .unwrap_err()
        .to_string();
    for fragment in expected {
        assert!(
            error.contains(fragment),
            "diagnostic omitted {fragment}: {error}"
        );
    }
}
