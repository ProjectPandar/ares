use super::*;

#[test]
fn generated_id_missing_variant_option_returns_minus_one_before_validation() {
    assert_eq!(
        options(json!({
            "printer_extruder_id": "bad",
            "extruder_variant_list": 7
        }))
        .get_index_for_extruder_generated_id_map(complete_lookup(
            1,
            ("printer_extruder_id", "printer_extruder_variant"),
            ("Cartesian", "Ultra Flow"),
            1,
        ))
        .unwrap(),
        -1
    );
}

#[test]
fn generated_id_ignores_short_id_values_and_uses_generated_ids() {
    assert_eq!(
        options(json!({
            "printer_extruder_id": [99],
            "printer_extruder_variant": ["Bowden Standard", "Direct Drive Standard"],
            "extruder_variant_list": ["Bowden Standard", "Direct Drive Standard"]
        }))
        .get_index_for_extruder_generated_id_map(complete_lookup(
            2,
            ("printer_extruder_id", "printer_extruder_variant"),
            ("Direct Drive", "Standard"),
            3,
        ))
        .unwrap(),
        3
    );
}

#[test]
fn generated_id_returns_extruder_index_plus_one_from_variant_token_order() {
    assert_eq!(
        options(json!({
            "printer_extruder_id": [1],
            "printer_extruder_variant": ["Bowden Standard", "Direct Drive Standard", "Direct Drive High Flow"],
            "extruder_variant_list": ["Bowden Standard,Direct Drive Standard", "Direct Drive High Flow"]
        }))
        .get_index_for_extruder_generated_id_map(complete_lookup(
            2,
            ("printer_extruder_id", "printer_extruder_variant"),
            ("Direct Drive", "High Flow"),
            4,
        ))
        .unwrap(),
        8
    );
}

#[test]
fn generated_id_missing_variant_list_returns_zero_id() {
    assert_eq!(
        options(json!({
            "printer_extruder_id": [1],
            "printer_extruder_variant": ["Bowden Standard", "Direct Drive Standard"]
        }))
        .get_index_for_extruder_generated_id_map(complete_lookup(
            0,
            ("printer_extruder_id", "printer_extruder_variant"),
            ("Direct Drive", "Standard"),
            5,
        ))
        .unwrap(),
        5
    );
}

#[test]
fn generated_id_missing_variant_list_nonzero_target_returns_minus_one() {
    assert_eq!(
        options(json!({
            "printer_extruder_id": [1],
            "printer_extruder_variant": ["Direct Drive Standard", "Bowden Standard"]
        }))
        .get_index_for_extruder_generated_id_map(complete_lookup(
            1,
            ("printer_extruder_id", "printer_extruder_variant"),
            ("Direct Drive", "Standard"),
            5,
        ))
        .unwrap(),
        -1
    );
}

#[test]
fn generated_id_target_beyond_generated_tokens_returns_zero_id() {
    assert_eq!(
        options(json!({
            "printer_extruder_id": [1],
            "printer_extruder_variant": ["Bowden Standard", "Direct Drive Standard"],
            "extruder_variant_list": ["Bowden Standard"]
        }))
        .get_index_for_extruder_generated_id_map(complete_lookup(
            0,
            ("printer_extruder_id", "printer_extruder_variant"),
            ("Direct Drive", "Standard"),
            7,
        ))
        .unwrap(),
        7
    );
}

#[test]
fn generated_id_split_trim_and_empty_skip_edge_cases() {
    assert_eq!(
        options(json!({
            "printer_extruder_id": [1],
            "printer_extruder_variant": ["Bowden Standard", "Direct Drive Standard", "Direct Drive High Flow"],
            "extruder_variant_list": [", Bowden Standard ,,", "  Direct Drive Standard  , ,Direct Drive High Flow,"]
        }))
        .get_index_for_extruder_generated_id_map(complete_lookup(
            2,
            ("printer_extruder_id", "printer_extruder_variant"),
            ("Direct Drive", "High Flow"),
            2,
        ))
        .unwrap(),
        4
    );
}

#[test]
fn generated_id_representative_option_names_match_index_times_stride() {
    for (id_name, variant_name) in [
        ("printer_extruder_id", "printer_extruder_variant"),
        ("print_extruder_id", "print_extruder_variant"),
        ("filament_self_index", "filament_extruder_variant"),
    ] {
        assert_eq!(
            options(json!({
                id_name: [1],
                variant_name: ["Bowden Standard", "Direct Drive Standard"],
                "extruder_variant_list": ["Bowden Standard", "Direct Drive Standard"]
            }))
            .get_index_for_extruder_generated_id_map(complete_lookup(
                2,
                (id_name, variant_name),
                ("Direct Drive", "Standard"),
                6,
            ))
            .unwrap(),
            6,
            "{id_name} {variant_name}"
        );
    }
}

#[test]
fn generated_id_duplicate_variants_continue_until_generated_id_matches() {
    assert_eq!(
        options(json!({
            "printer_extruder_id": [1],
            "printer_extruder_variant": ["Direct Drive Standard", "Direct Drive Standard"],
            "extruder_variant_list": ["Bowden Standard", "Direct Drive Standard"]
        }))
        .get_index_for_extruder_generated_id_map(complete_lookup(
            2,
            ("printer_extruder_id", "printer_extruder_variant"),
            ("Direct Drive", "Standard"),
            9,
        ))
        .unwrap(),
        9
    );
}

#[test]
fn generated_id_no_pair_match_returns_minus_one() {
    assert_eq!(
        options(json!({
            "printer_extruder_id": [1],
            "printer_extruder_variant": ["Direct Drive Standard", "Bowden Standard"],
            "extruder_variant_list": ["Direct Drive Standard"]
        }))
        .get_index_for_extruder_generated_id_map(complete_lookup(
            2,
            ("printer_extruder_id", "printer_extruder_variant"),
            ("Direct Drive", "Standard"),
            1,
        ))
        .unwrap(),
        -1
    );
}

#[test]
fn generated_id_valid_enum_combinations_generate_source_variant_strings() {
    for (extruder_type, nozzle_volume_type, variant) in [
        ("Direct Drive", "Standard", "Direct Drive Standard"),
        ("Direct Drive", "High Flow", "Direct Drive High Flow"),
        ("Bowden", "Standard", "Bowden Standard"),
        ("Bowden", "High Flow", "Bowden High Flow"),
    ] {
        assert_eq!(
            options(json!({
                "printer_extruder_id": [1],
                "printer_extruder_variant": ["PLA", variant],
                "extruder_variant_list": ["PLA", variant]
            }))
            .get_index_for_extruder_generated_id_map(complete_lookup(
                2,
                ("printer_extruder_id", "printer_extruder_variant"),
                (extruder_type, nozzle_volume_type),
                3,
            ))
            .unwrap(),
            3,
            "{variant}"
        );
    }
}

#[test]
fn generated_id_zero_stride_returns_zero_for_match() {
    assert_eq!(
        options(json!({
            "printer_extruder_id": [1],
            "printer_extruder_variant": ["Bowden Standard", "Direct Drive Standard"],
            "extruder_variant_list": ["Bowden Standard", "Direct Drive Standard"]
        }))
        .get_index_for_extruder_generated_id_map(complete_lookup(
            2,
            ("printer_extruder_id", "printer_extruder_variant"),
            ("Direct Drive", "Standard"),
            0,
        ))
        .unwrap(),
        0
    );
}

#[test]
fn generated_id_overflowing_index_stride_returns_invalid_input() {
    assert!(matches!(
        options(json!({
            "printer_extruder_id": [1],
            "printer_extruder_variant": ["Bowden Standard", "Direct Drive Standard"],
            "extruder_variant_list": ["Bowden Standard", "Direct Drive Standard"]
        }))
        .get_index_for_extruder_generated_id_map(complete_lookup(
            2,
            ("printer_extruder_id", "printer_extruder_variant"),
            ("Direct Drive", "Standard"),
            usize::MAX,
        )),
        Err(SliceError::InvalidInput(_))
    ));
}

#[test]
fn generated_id_invalid_boundary_values_return_invalid_input() {
    for value in [
        json!({ "printer_extruder_id": [1], "printer_extruder_variant": "Direct Drive Standard", "extruder_variant_list": ["Direct Drive Standard"] }),
        json!({ "printer_extruder_id": [1], "printer_extruder_variant": [], "extruder_variant_list": ["Direct Drive Standard"] }),
        json!({ "printer_extruder_id": [1], "printer_extruder_variant": [7], "extruder_variant_list": ["Direct Drive Standard"] }),
        json!({ "printer_extruder_variant": ["Direct Drive Standard"], "extruder_variant_list": ["Direct Drive Standard"] }),
        json!({ "printer_extruder_id": 1, "printer_extruder_variant": ["Direct Drive Standard"], "extruder_variant_list": ["Direct Drive Standard"] }),
        json!({ "printer_extruder_id": [], "printer_extruder_variant": ["Direct Drive Standard"], "extruder_variant_list": ["Direct Drive Standard"] }),
        json!({ "printer_extruder_id": [1, 2], "printer_extruder_variant": ["Direct Drive Standard"], "extruder_variant_list": ["Direct Drive Standard"] }),
        json!({ "printer_extruder_id": ["1"], "printer_extruder_variant": ["Direct Drive Standard"], "extruder_variant_list": ["Direct Drive Standard"] }),
        json!({ "printer_extruder_id": [i64::from(i32::MAX) + 1], "printer_extruder_variant": ["Direct Drive Standard"], "extruder_variant_list": ["Direct Drive Standard"] }),
        json!({ "printer_extruder_id": [1], "printer_extruder_variant": ["Direct Drive Standard"], "extruder_variant_list": "Direct Drive Standard" }),
        json!({ "printer_extruder_id": [1], "printer_extruder_variant": ["Direct Drive Standard"], "extruder_variant_list": [] }),
        json!({ "printer_extruder_id": [1], "printer_extruder_variant": ["Direct Drive Standard"], "extruder_variant_list": [7] }),
    ] {
        assert!(matches!(
            options(value).get_index_for_extruder_generated_id_map(complete_lookup(
                1,
                ("printer_extruder_id", "printer_extruder_variant"),
                ("Direct Drive", "Standard"),
                1,
            )),
            Err(SliceError::InvalidInput(_))
        ));
    }
}

#[test]
fn generated_id_unknown_enum_values_return_invalid_input() {
    for (extruder_type, nozzle_volume_type) in
        [("Cartesian", "Standard"), ("Direct Drive", "Ultra Flow")]
    {
        assert!(matches!(
            options(json!({
                "printer_extruder_id": [1],
                "printer_extruder_variant": ["Direct Drive Standard"],
                "extruder_variant_list": ["Direct Drive Standard"]
            }))
            .get_index_for_extruder_generated_id_map(complete_lookup(
                1,
                ("printer_extruder_id", "printer_extruder_variant"),
                (extruder_type, nozzle_volume_type),
                1,
            )),
            Err(SliceError::InvalidInput(_))
        ));
    }
}
