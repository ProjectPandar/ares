use super::*;
#[test]
fn missing_variant_option_returns_minus_one() {
    for (extruder_type, nozzle_volume_type) in
        [("Direct Drive", "Standard"), ("Cartesian", "Ultra Flow")]
    {
        assert_eq!(
            options(json!({}))
                .get_index_for_extruder_no_id(
                    extruder_type,
                    nozzle_volume_type,
                    "printer_extruder_variant",
                    1
                )
                .unwrap(),
            -1
        );
    }
}

#[test]
fn first_match_returns_index_times_stride() {
    assert_eq!(
        options(json!({
            "printer_extruder_variant": ["Bowden Standard", "Direct Drive Standard"]
        }))
        .get_index_for_extruder_no_id("Direct Drive", "Standard", "printer_extruder_variant", 3)
        .unwrap(),
        3
    );
}

#[test]
fn duplicate_variants_return_first_match() {
    assert_eq!(
        options(json!({
            "printer_extruder_variant": ["Direct Drive Standard", "Direct Drive Standard"]
        }))
        .get_index_for_extruder_no_id("Direct Drive", "Standard", "printer_extruder_variant", 5)
        .unwrap(),
        0
    );
}

#[test]
fn no_match_returns_minus_one() {
    assert_eq!(
        options(json!({
            "printer_extruder_variant": ["Bowden Standard"]
        }))
        .get_index_for_extruder_no_id("Direct Drive", "High Flow", "printer_extruder_variant", 2)
        .unwrap(),
        -1
    );
}

#[test]
fn valid_enum_combinations_generate_source_variant_strings() {
    for (extruder_type, nozzle_volume_type, variant) in [
        ("Direct Drive", "Standard", "Direct Drive Standard"),
        ("Direct Drive", "High Flow", "Direct Drive High Flow"),
        ("Bowden", "Standard", "Bowden Standard"),
        ("Bowden", "High Flow", "Bowden High Flow"),
    ] {
        assert_eq!(
            options(json!({ "filament_extruder_variant": [variant] }))
                .get_index_for_extruder_no_id(
                    extruder_type,
                    nozzle_volume_type,
                    "filament_extruder_variant",
                    7,
                )
                .unwrap(),
            0,
            "{variant}"
        );
    }
}

#[test]
fn zero_stride_returns_zero_for_match() {
    assert_eq!(
        options(json!({
            "printer_extruder_variant": ["Bowden Standard", "Direct Drive Standard"]
        }))
        .get_index_for_extruder_no_id("Direct Drive", "Standard", "printer_extruder_variant", 0)
        .unwrap(),
        0
    );
}

#[test]
fn overflowing_index_stride_returns_invalid_input() {
    assert!(matches!(
        options(json!({
            "printer_extruder_variant": ["Bowden Standard", "Direct Drive Standard"]
        }))
        .get_index_for_extruder_no_id(
            "Direct Drive",
            "Standard",
            "printer_extruder_variant",
            usize::MAX,
        ),
        Err(SliceError::InvalidInput(_))
    ));
}

#[test]
fn invalid_variant_boundary_values_return_invalid_input() {
    for value in [
        json!({ "printer_extruder_variant": "Direct Drive Standard" }),
        json!({ "printer_extruder_variant": [] }),
        json!({ "printer_extruder_variant": [7] }),
    ] {
        assert!(matches!(
            options(value).get_index_for_extruder_no_id(
                "Direct Drive",
                "Standard",
                "printer_extruder_variant",
                1,
            ),
            Err(SliceError::InvalidInput(_))
        ));
    }
}

#[test]
fn unknown_enum_values_return_invalid_input() {
    for (extruder_type, nozzle_volume_type) in
        [("Cartesian", "Standard"), ("Direct Drive", "Ultra Flow")]
    {
        assert!(matches!(
            options(json!({ "printer_extruder_variant": ["Direct Drive Standard"] }))
                .get_index_for_extruder_no_id(
                    extruder_type,
                    nozzle_volume_type,
                    "printer_extruder_variant",
                    1,
                ),
            Err(SliceError::InvalidInput(_))
        ));
    }
}
