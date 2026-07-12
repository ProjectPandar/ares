use super::*;
#[test]
fn complete_id_missing_variant_option_returns_minus_one_before_validation() {
    assert_eq!(
        options(json!({ "printer_extruder_id": "bad" }))
            .get_index_for_extruder_complete_id_map(complete_lookup(
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
fn complete_id_representative_option_names_match_index_times_stride() {
    for (id_name, variant_name) in [
        ("printer_extruder_id", "printer_extruder_variant"),
        ("print_extruder_id", "print_extruder_variant"),
        ("filament_self_index", "filament_extruder_variant"),
    ] {
        assert_eq!(
            options(json!({
                id_name: [4, 7],
                variant_name: ["Bowden Standard", "Direct Drive Standard"]
            }))
            .get_index_for_extruder_complete_id_map(complete_lookup(
                7,
                (id_name, variant_name),
                ("Direct Drive", "Standard"),
                3,
            ))
            .unwrap(),
            3,
            "{id_name} {variant_name}"
        );
    }
}

#[test]
fn complete_id_variant_match_with_wrong_id_continues_searching() {
    assert_eq!(
        options(json!({
            "printer_extruder_id": [1, 2],
            "printer_extruder_variant": ["Direct Drive Standard", "Direct Drive Standard"]
        }))
        .get_index_for_extruder_complete_id_map(complete_lookup(
            2,
            ("printer_extruder_id", "printer_extruder_variant"),
            ("Direct Drive", "Standard"),
            5,
        ))
        .unwrap(),
        5
    );
}

#[test]
fn complete_id_duplicate_variants_return_first_matching_id() {
    assert_eq!(
        options(json!({
            "printer_extruder_id": [2, 2],
            "printer_extruder_variant": ["Direct Drive Standard", "Direct Drive Standard"]
        }))
        .get_index_for_extruder_complete_id_map(complete_lookup(
            2,
            ("printer_extruder_id", "printer_extruder_variant"),
            ("Direct Drive", "Standard"),
            5,
        ))
        .unwrap(),
        0
    );
}

#[test]
fn complete_id_no_pair_match_returns_minus_one() {
    assert_eq!(
        options(json!({
            "printer_extruder_id": [1, 2],
            "printer_extruder_variant": ["Direct Drive Standard", "Bowden Standard"]
        }))
        .get_index_for_extruder_complete_id_map(complete_lookup(
            3,
            ("printer_extruder_id", "printer_extruder_variant"),
            ("Direct Drive", "Standard"),
            2,
        ))
        .unwrap(),
        -1
    );
}

#[test]
fn complete_id_valid_enum_combinations_generate_source_variant_strings() {
    for (extruder_type, nozzle_volume_type, variant) in [
        ("Direct Drive", "Standard", "Direct Drive Standard"),
        ("Direct Drive", "High Flow", "Direct Drive High Flow"),
        ("Bowden", "Standard", "Bowden Standard"),
        ("Bowden", "High Flow", "Bowden High Flow"),
    ] {
        assert_eq!(
            options(json!({
                "printer_extruder_id": [9],
                "printer_extruder_variant": [variant]
            }))
            .get_index_for_extruder_complete_id_map(complete_lookup(
                9,
                ("printer_extruder_id", "printer_extruder_variant"),
                (extruder_type, nozzle_volume_type),
                7,
            ))
            .unwrap(),
            0,
            "{variant}"
        );
    }
}

#[test]
fn complete_id_zero_stride_returns_zero_for_match() {
    assert_eq!(
        options(json!({
            "printer_extruder_id": [1, 2],
            "printer_extruder_variant": ["Bowden Standard", "Direct Drive Standard"]
        }))
        .get_index_for_extruder_complete_id_map(complete_lookup(
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
fn complete_id_overflowing_index_stride_returns_invalid_input() {
    assert!(matches!(
        options(json!({
            "printer_extruder_id": [1, 2],
            "printer_extruder_variant": ["Bowden Standard", "Direct Drive Standard"]
        }))
        .get_index_for_extruder_complete_id_map(complete_lookup(
            2,
            ("printer_extruder_id", "printer_extruder_variant"),
            ("Direct Drive", "Standard"),
            usize::MAX,
        )),
        Err(SliceError::InvalidInput(_))
    ));
}

#[test]
fn complete_id_invalid_variant_boundary_values_return_invalid_input() {
    for value in [
        json!({ "printer_extruder_id": [1], "printer_extruder_variant": "Direct Drive Standard" }),
        json!({ "printer_extruder_id": [1], "printer_extruder_variant": [] }),
        json!({ "printer_extruder_id": [1], "printer_extruder_variant": [7] }),
    ] {
        assert!(matches!(
            options(value).get_index_for_extruder_complete_id_map(complete_lookup(
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
fn complete_id_invalid_id_boundary_values_return_invalid_input() {
    for value in [
        json!({ "printer_extruder_variant": ["Direct Drive Standard"] }),
        json!({ "printer_extruder_id": 1, "printer_extruder_variant": ["Direct Drive Standard"] }),
        json!({ "printer_extruder_id": [], "printer_extruder_variant": ["Direct Drive Standard"] }),
        json!({ "printer_extruder_id": [1], "printer_extruder_variant": ["Direct Drive Standard", "Bowden Standard"] }),
        json!({ "printer_extruder_id": ["1"], "printer_extruder_variant": ["Direct Drive Standard"] }),
        json!({ "printer_extruder_id": [i64::from(i32::MAX) + 1], "printer_extruder_variant": ["Direct Drive Standard"] }),
    ] {
        assert!(matches!(
            options(value).get_index_for_extruder_complete_id_map(complete_lookup(
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
fn complete_id_unknown_enum_values_return_invalid_input() {
    for (extruder_type, nozzle_volume_type) in
        [("Cartesian", "Standard"), ("Direct Drive", "Ultra Flow")]
    {
        assert!(matches!(
            options(json!({
                "printer_extruder_id": [1],
                "printer_extruder_variant": ["Direct Drive Standard"]
            }))
            .get_index_for_extruder_complete_id_map(complete_lookup(
                1,
                ("printer_extruder_id", "printer_extruder_variant"),
                (extruder_type, nozzle_volume_type),
                1,
            )),
            Err(SliceError::InvalidInput(_))
        ));
    }
}
