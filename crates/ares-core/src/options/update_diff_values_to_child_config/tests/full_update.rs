use super::*;

#[test]
fn diff_child_update_assembly_directly_copies_scalar_and_non_restore_vector() {
    let mut current = options(&[
        ("printer_extruder_variant", json!(["a"])),
        ("speed", json!(40)),
        ("colors", json!(["red"])),
    ]);
    let target = options(&[
        ("printer_extruder_variant", json!(["a"])),
        ("speed", json!(60)),
        ("colors", json!(["blue", "green"])),
    ]);

    apply_diff_values_to_child_config(
        &mut current,
        &target,
        DiffChildConfigKeys {
            keys: &["speed", "colors"],
            extruder_id_name: "printer_extruder_id",
            extruder_variant_name: "printer_extruder_variant",
            key_set1: &[],
            key_set2: &[],
        },
    )
    .unwrap();

    assert_eq!(current.values().get("speed"), Some(&json!(60)));
    assert_eq!(
        current.values().get("colors"),
        Some(&json!(["blue", "green"]))
    );
}

#[test]
fn diff_child_update_assembly_applies_stride1_restore_vectors() {
    let mut current = options(&[
        ("printer_extruder_variant", json!(["a", "b", "c"])),
        ("temperature", json!([200, 210, 220])),
    ]);
    let target = options(&[
        ("printer_extruder_variant", json!(["c", "a", "b"])),
        ("temperature", json!([220, 200, 210])),
    ]);

    apply_diff_values_to_child_config(
        &mut current,
        &target,
        DiffChildConfigKeys {
            keys: &["temperature"],
            extruder_id_name: "printer_extruder_id",
            extruder_variant_name: "printer_extruder_variant",
            key_set1: &["temperature"],
            key_set2: &[],
        },
    )
    .unwrap();

    assert_eq!(
        current.values().get("temperature"),
        Some(&json!([200, 210, 220]))
    );
}

#[test]
fn diff_child_update_assembly_applies_stride2_restore_vectors() {
    let mut current = options(&[
        ("printer_extruder_variant", json!(["a", "b", "c"])),
        ("machine_limits", json!([10, 11, 20, 21, 30, 31])),
    ]);
    let target = options(&[
        ("printer_extruder_variant", json!(["c", "a", "b"])),
        ("machine_limits", json!([30, 31, 10, 11, 20, 21])),
    ]);

    apply_diff_values_to_child_config(
        &mut current,
        &target,
        DiffChildConfigKeys {
            keys: &["machine_limits"],
            extruder_id_name: "printer_extruder_id",
            extruder_variant_name: "printer_extruder_variant",
            key_set1: &[],
            key_set2: &["machine_limits"],
        },
    )
    .unwrap();

    assert_eq!(
        current.values().get("machine_limits"),
        Some(&json!([10, 11, 20, 21, 30, 31]))
    );
}

#[test]
fn diff_child_update_assembly_skips_metadata_missing_and_equal_keys() {
    let mut current = options(&[
        ("printer_extruder_id", json!([1])),
        ("printer_extruder_variant", json!(["a"])),
        ("equal", json!(1)),
        ("missing_target", json!(2)),
    ]);
    let target = options(&[
        ("printer_extruder_id", json!([2])),
        ("printer_extruder_variant", json!(["b"])),
        ("equal", json!(1)),
        ("missing_source", json!(3)),
    ]);

    apply_diff_values_to_child_config(
        &mut current,
        &target,
        DiffChildConfigKeys {
            keys: &[
                "printer_extruder_id",
                "printer_extruder_variant",
                "equal",
                "missing_target",
                "missing_source",
            ],
            extruder_id_name: "printer_extruder_id",
            extruder_variant_name: "printer_extruder_variant",
            key_set1: &[],
            key_set2: &[],
        },
    )
    .unwrap();

    assert_eq!(
        current.values().get("printer_extruder_id"),
        Some(&json!([1]))
    );
    assert_eq!(
        current.values().get("printer_extruder_variant"),
        Some(&json!(["a"]))
    );
    assert_eq!(current.values().get("equal"), Some(&json!(1)));
    assert_eq!(current.values().get("missing_target"), Some(&json!(2)));
    assert!(!current.values().contains_key("missing_source"));
}

#[test]
fn diff_child_update_assembly_skips_null_target_restore_slot() {
    let mut current = options(&[
        ("printer_extruder_variant", json!(["a", "b"])),
        ("temperature", json!([200, 210])),
    ]);
    let target = options(&[
        ("printer_extruder_variant", json!(["b", "a"])),
        ("temperature", json!([220, null])),
    ]);

    apply_diff_values_to_child_config(
        &mut current,
        &target,
        DiffChildConfigKeys {
            keys: &["temperature"],
            extruder_id_name: "printer_extruder_id",
            extruder_variant_name: "printer_extruder_variant",
            key_set1: &["temperature"],
            key_set2: &[],
        },
    )
    .unwrap();

    assert_eq!(
        current.values().get("temperature"),
        Some(&json!([200, 220]))
    );
}
