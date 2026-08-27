use serde_json::{Value, json};

use crate::options::{
    ProjectSettings,
    project_variants::{inspect_printer_indices_for_test, materialize_project_variants},
    registry::{
        print_options_with_variant, printer_options_with_variant_1,
        printer_options_with_variant_2,
    },
};

use super::support::{
    active_source, flat_settings, ints, source_with_overrides,
};

#[test]
fn printer_variant_one_and_process_select_exact_base_indices() {
    let source = sentinel_source();
    let original = source.clone();

    assert_eq!(
        inspect_printer_indices_for_test(&source).unwrap().unwrap(),
        [0, 2]
    );
    let materialized = materialize_project_variants(&source, &ints(&[1, 2])).unwrap();

    assert_eq!(source, original);
    assert_eq!(
        materialized.printer.gcode.printer_extruder_id,
        ints(&[1, 2])
    );
    assert_eq!(
        materialized.printer.gcode.printer_extruder_variant.0,
        ["Direct Drive Standard", "Bowden Standard"]
    );
    assert_eq!(
        materialized.process.region.print_extruder_id,
        ints(&[1, 2])
    );
    assert_eq!(
        materialized.process.region.print_extruder_variant.0,
        ["Direct Drive Standard", "Bowden Standard"]
    );
    assert_eq!(
        materialized.printer.gcode.nozzle_flush_dataset.0,
        selected(&source.printer.gcode.nozzle_flush_dataset.0)
    );
    assert_eq!(
        materialized.printer.gcode.nozzle_type.0,
        selected(&source.printer.gcode.nozzle_type.0)
    );
    assert_eq!(
        materialized.printer.gcode.retract_lift_enforce.0,
        selected(&source.printer.gcode.retract_lift_enforce.0)
    );
    assert_eq!(
        materialized.printer.gcode.z_hop_types.0,
        selected(&source.printer.gcode.z_hop_types.0)
    );

    assert_eq!(printer_options_with_variant_1().len(), 24);
    assert_eq!(print_options_with_variant().len(), 2);
    assert_selected_keys(
        &source,
        &materialized,
        printer_options_with_variant_1(),
    );
    assert_selected_keys(&source, &materialized, print_options_with_variant());
}

#[test]
fn printer_variant_two_reresolves_after_variant_one_and_selects_all_machine_limits() {
    let source = sentinel_source();
    let original = source.clone();

    assert_eq!(
        inspect_printer_indices_for_test(&source).unwrap().unwrap(),
        [0, 2]
    );
    let materialized = materialize_project_variants(&source, &ints(&[1, 2])).unwrap();

    assert_eq!(source, original);
    assert_eq!(
        inspect_printer_indices_for_test(&materialized)
            .unwrap()
            .unwrap(),
        [0, 1]
    );
    assert_eq!(
        materialized
            .printer
            .machine
            .machine_max_acceleration_e
            .0,
        source.printer.machine.machine_max_acceleration_e.0[..4]
    );
    assert_eq!(printer_options_with_variant_2().len(), 15);
    assert_stride_two_selected_keys(
        &source,
        &materialized,
        printer_options_with_variant_2(),
    );
}

#[test]
fn short_printer_payloads_broadcast_the_first_value() {
    let mut source = sentinel_source();
    source
        .printer
        .machine
        .machine_max_acceleration_e
        .0
        .truncate(2);
    source.project.gcode.retraction_length.0.truncate(2);

    let materialized = materialize_project_variants(&source, &ints(&[1, 2])).unwrap();

    assert_eq!(
        materialized.printer.machine.machine_max_acceleration_e.0,
        [
            source.printer.machine.machine_max_acceleration_e.0[0],
            source.printer.machine.machine_max_acceleration_e.0[1],
            source.printer.machine.machine_max_acceleration_e.0[0],
            source.printer.machine.machine_max_acceleration_e.0[0],
        ]
    );
    assert_eq!(
        materialized.project.gcode.retraction_length.0,
        [
            source.project.gcode.retraction_length.0[0],
            source.project.gcode.retraction_length.0[0],
        ]
    );
}

#[test]
fn incomplete_process_ids_are_generated_from_variant_groups() {
    let mut source = sentinel_source();
    source.printer.remaining.extruder_variant_list.0 = vec![
        "Direct Drive Standard,unused".to_owned(),
        "Bowden Standard".to_owned(),
    ];
    source.process.region.print_extruder_id = ints(&[1, 1]);

    let materialized = materialize_project_variants(&source, &ints(&[1, 2])).unwrap();

    assert_eq!(materialized.process.region.print_extruder_id, ints(&[1, 1]));
    assert_eq!(
        materialized.process.region.print_extruder_variant.0,
        ["Direct Drive Standard", "Bowden Standard"]
    );
}

fn sentinel_source() -> ProjectSettings {
    let source = source_with_overrides(
        &active_source(),
        json!({
            "deretraction_speed": [101, 102, 103],
            "long_retractions_when_cut": [true, false, true],
            "nozzle_flush_dataset": ["nil", 1202, 1203],
            "nozzle_type": ["stainless_steel", "brass", "hardened_steel"],
            "nozzle_volume": [1301, 1302, 1303],
            "printer_extruder_id": [1, 9, 2],
            "printer_extruder_variant": [
                "Direct Drive Standard",
                "unused",
                "Bowden Standard"
            ],
            "retract_before_wipe": ["21%", "22%", "23%"],
            "retract_length_toolchange": [301, 302, 303],
            "retract_lift_above": [401, 402, 403],
            "retract_lift_below": [501, 502, 503],
            "retract_lift_enforce": ["All Surfaces", "Top Only", "Bottom Only"],
            "retract_restart_extra": [601, 602, 603],
            "retract_restart_extra_toolchange": [701, 702, 703],
            "retract_when_changing_layer": [true, true, false],
            "retraction_distances_when_cut": [801, 802, 803],
            "retraction_length": [901, 902, 903],
            "retraction_minimum_travel": [1001, 1002, 1003],
            "retraction_speed": [1101, 1102, 1103],
            "travel_slope": [1201, 1202, 1203],
            "wipe": [false, true, true],
            "wipe_distance": [1301, 1302, 1303],
            "z_hop": [1401, 1402, 1403],
            "z_hop_types": ["Auto Lift", "Normal Lift", "Spiral Lift"],
            "print_extruder_id": [1, 9, 2],
            "print_extruder_variant": [
                "Direct Drive Standard",
                "unused",
                "Bowden Standard"
            ]
        }),
    );
    source_with_overrides(
        &source,
        json!({
            "machine_max_acceleration_e": [10, 11, 20, 21, 30, 31],
            "machine_max_acceleration_extruding": [110, 111, 120, 121, 130, 131],
            "machine_max_acceleration_retracting": [210, 211, 220, 221, 230, 231],
            "machine_max_acceleration_travel": [310, 311, 320, 321, 330, 331],
            "machine_max_acceleration_x": [410, 411, 420, 421, 430, 431],
            "machine_max_acceleration_y": [510, 511, 520, 521, 530, 531],
            "machine_max_acceleration_z": [610, 611, 620, 621, 630, 631],
            "machine_max_jerk_e": [710, 711, 720, 721, 730, 731],
            "machine_max_jerk_x": [810, 811, 820, 821, 830, 831],
            "machine_max_jerk_y": [910, 911, 920, 921, 930, 931],
            "machine_max_jerk_z": [1010, 1011, 1020, 1021, 1030, 1031],
            "machine_max_speed_e": [1110, 1111, 1120, 1121, 1130, 1131],
            "machine_max_speed_x": [1210, 1211, 1220, 1221, 1230, 1231],
            "machine_max_speed_y": [1310, 1311, 1320, 1321, 1330, 1331],
            "machine_max_speed_z": [1410, 1411, 1420, 1421, 1430, 1431]
        }),
    )
}

fn selected<T: Clone>(values: &[T]) -> Vec<T> {
    vec![values[0].clone(), values[2].clone()]
}

fn assert_selected_keys(source: &ProjectSettings, materialized: &ProjectSettings, keys: &[&str]) {
    let source = flat_settings(source);
    let materialized = flat_settings(materialized);
    for key in keys {
        let values = source[*key].as_array().unwrap();
        let expected = Value::Array(vec![values[0].clone(), values[2].clone()]);
        assert_eq!(materialized[*key], expected, "wrong selection for {key}");
        assert_eq!(materialized[*key].as_array().unwrap().len(), 2, "{key}");
    }
}

fn assert_stride_two_selected_keys(
    source: &ProjectSettings,
    materialized: &ProjectSettings,
    keys: &[&str],
) {
    let source = flat_settings(source);
    let materialized = flat_settings(materialized);
    for key in keys {
        let values = source[*key].as_array().unwrap();
        let expected = Value::Array(values[..4].to_vec());
        assert_eq!(materialized[*key], expected, "wrong selection for {key}");
        assert_eq!(materialized[*key].as_array().unwrap().len(), 4, "{key}");
    }
}
