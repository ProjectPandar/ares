use super::{array, assert_assign, assert_invalid_array};
use crate::options::typed_legacy::{
    EXPLICIT_RULES, JsonArrayAllowance, LegacyOutcome, array_first_pass,
};

#[test]
fn all_twelve_registered_array_sources_flatten_to_their_canonical_targets() {
    let cases = [
        ("bridge_fan_speed", "[\"7\"]", "overhang_fan_speed", "7"),
        ("chamber_temperatures", "[\"45\"]", "chamber_temperature", "45"),
        ("cooling", "[\"1\"]", "slow_down_for_layer_cooling", "1"),
        ("nozzle_volume_type", "[\"Normal\"]", "nozzle_volume_type", "Standard"),
        ("default_nozzle_volume_type", "[\"Normal\"]", "default_nozzle_volume_type", "Standard"),
        ("extruder_type", "[\"DirectDrive\"]", "extruder_type", "Direct Drive"),
        ("overhang_fan_threshold", "[\"5%\"]", "overhang_fan_threshold", "10%"),
        ("extruder_variant_list", "[\"Normal\"]", "extruder_variant_list", "\"Standard\""),
        ("filament_extruder_variant", "[\"Normal\"]", "filament_extruder_variant", "\"Standard\""),
        ("filament_type", "[\"ASA-Aero\"]", "filament_type", "\"ASA-AERO\""),
        ("print_extruder_variant", "[\"Normal\"]", "print_extruder_variant", "\"Standard\""),
        ("printer_extruder_variant", "[\"Normal\"]", "printer_extruder_variant", "\"Standard\""),
    ];

    for (source, json, target, value) in cases {
        assert_assign(array(source, json), target, value);
    }
}

#[test]
fn fixed_vector_shapes_use_comma_or_quoted_semicolon_flattening() {
    assert_assign(
        array("bridge_fan_speed", "[\"7\",\"8\"]"),
        "overhang_fan_speed",
        "7,8",
    );
    assert_assign(
        array("cooling", "[\"1\",\"0\"]"),
        "slow_down_for_layer_cooling",
        "1,0",
    );
    assert_assign(
        array("printer_extruder_variant", r#"["a\"b","c\\d"]"#),
        "printer_extruder_variant",
        r#""a\"b";"c\\d""#,
    );
    assert_assign(
        array(
            "printer_extruder_variant",
            r#"["line\nend","carriage\rreturn","a\tb"]"#,
        ),
        "printer_extruder_variant",
        "\"line\\nend\";\"carriage\\rreturn\";\"a\tb\"",
    );
}

#[test]
fn homogeneous_two_depth_arrays_use_hash_between_groups() {
    assert_assign(
        array("bridge_fan_speed", r#"[["1","2"],["3"]]"#),
        "overhang_fan_speed",
        "1,2#3",
    );
    assert_assign(
        array(
            "printer_extruder_variant",
            r#"[["Normal","x"],["Big Traffic"]]"#,
        ),
        "printer_extruder_variant",
        "\"Standard\";\"x\"#\"High Flow\"",
    );
}

#[test]
fn second_pass_transforms_the_complete_flattened_string() {
    assert_assign(
        array("nozzle_volume_type", "[\"Normal\",\"Big Traffic\"]"),
        "nozzle_volume_type",
        "Standard,High Flow",
    );
    assert_assign(
        array(
            "default_nozzle_volume_type",
            "[\"Normal\",\"Big Traffic\"]",
        ),
        "default_nozzle_volume_type",
        "Standard,High Flow",
    );
    assert_assign(
        array("extruder_type", "[\"DirectDrive\",\"Bowden\"]"),
        "extruder_type",
        "Direct Drive,Bowden",
    );
    assert_assign(
        array(
            "extruder_variant_list",
            "[\"Normal\",\"Big Traffic\"]",
        ),
        "extruder_variant_list",
        "\"Standard\";\"High Flow\"",
    );
    assert_assign(
        array(
            "filament_extruder_variant",
            "[\"Normal\",\"Big Traffic\"]",
        ),
        "filament_extruder_variant",
        "\"Standard\";\"High Flow\"",
    );
    assert_assign(
        array(
            "print_extruder_variant",
            "[\"Normal\",\"Big Traffic\"]",
        ),
        "print_extruder_variant",
        "\"Standard\";\"High Flow\"",
    );
    assert_assign(
        array("printer_extruder_variant", "[\"Normal\",\"Big Traffic\"]"),
        "printer_extruder_variant",
        "\"Standard\";\"High Flow\"",
    );
    assert_assign(
        array("filament_type", "[\"ASA-Aero\",\"PLA\"]"),
        "filament_type",
        "\"ASA-AERO\";\"PLA\"",
    );
    assert_assign(
        array("overhang_fan_threshold", "[\"5%\",\"10%\"]"),
        "overhang_fan_threshold",
        "5%,10%",
    );
}

#[test]
fn empty_first_pass_preserves_top_one_value_and_consumes_prime_rib_arrays() {
    assert_assign(
        array_first_pass(super::rule("top_one_wall_type")),
        "only_one_wall_top",
        "1",
    );
    assert_invalid_array(
        array("top_one_wall_type", "[\"top\"]"),
        "top_one_wall_type",
    );
    assert_eq!(
        array_first_pass(super::rule("prime_tower_rib_wall")),
        LegacyOutcome::Consume
    );
    assert_eq!(
        array("prime_tower_rib_wall", "[1,{\"nested\":[true]}]"),
        LegacyOutcome::Consume
    );
    assert_assign(
        array("bridge_fan_speed", "[]"),
        "overhang_fan_speed",
        "",
    );
}

#[test]
fn every_string_only_rule_rejects_json_arrays_after_its_first_pass() {
    for rule in EXPLICIT_RULES {
        if rule.wire.json_array == JsonArrayAllowance::RejectAfterFirstPass {
            assert_invalid_array(array(rule.source, "[\"value\"]"), rule.source);
        }
    }
}

#[test]
fn allowed_arrays_reject_non_string_mixed_nested_and_non_array_values() {
    let array_sources = [
        "bridge_fan_speed",
        "chamber_temperatures",
        "cooling",
        "nozzle_volume_type",
        "default_nozzle_volume_type",
        "extruder_type",
        "overhang_fan_threshold",
        "extruder_variant_list",
        "filament_extruder_variant",
        "filament_type",
        "print_extruder_variant",
        "printer_extruder_variant",
    ];
    for source in array_sources {
        assert_invalid_array(array(source, "[1]"), source);
    }

    for invalid in [
        "[\"1\",2]",
        "[\"1\",[\"2\"]]",
        "[[\"1\",2]]",
        "[[[\"1\"]]]",
        "{\"value\":[\"1\"]}",
        "\"1\"",
        "[\"1\"",
    ] {
        assert_invalid_array(array("bridge_fan_speed", invalid), "bridge_fan_speed");
    }
}
