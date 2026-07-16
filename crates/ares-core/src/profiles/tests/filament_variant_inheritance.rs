use std::{cell::Cell, collections::BTreeSet};

use crate::{
    FilamentGCodeSourceOptions, FilamentOptions, FilamentPrintSourceOptions,
    FilamentRegionSourceOptions, FilamentRetractOverrideOptions, MergedProfile, Nullable, OrcaBool,
    OrcaFloat, OrcaInt, Percent, ProfileFragment, ProfileKind, SliceError, merge_profile_fragments,
};

use super::{assert_invalid, fragments};

fn merge_options(items: &[ProfileFragment], target: &str) -> Result<FilamentOptions, SliceError> {
    match merge_profile_fragments(items, ProfileKind::Filament, target)? {
        MergedProfile::Filament { options, .. } => Ok(options),
        _ => unreachable!(),
    }
}

fn merged<const N: usize>(inputs: [&[u8]; N], target: &str) -> FilamentOptions {
    merge_options(&fragments(inputs), target).unwrap()
}

fn floats(values: &[Nullable<OrcaFloat>]) -> Vec<Option<f64>> {
    values
        .iter()
        .map(|value| match value {
            Nullable::Nil => None,
            Nullable::Value(value) => Some(value.0),
        })
        .collect()
}

fn percents(values: &[Nullable<Percent>]) -> Vec<Option<f64>> {
    values
        .iter()
        .map(|value| match value {
            Nullable::Nil => None,
            Nullable::Value(value) => Some(value.0),
        })
        .collect()
}

#[test]
fn exact_one_plus_thirty_six_inventory_is_concrete_owner_declared() {
    #[rustfmt::skip]
    const GCODE: [&str; 10] = [
        "filament_extruder_variant", "filament_flow_ratio", "filament_max_volumetric_speed",
        "long_retractions_when_ec", "retraction_distances_when_ec", "filament_flush_volumetric_speed",
        "filament_flush_temp", "filament_cooling_before_tower", "volumetric_speed_coefficients",
        "filament_adaptive_volumetric_speed",
    ];
    #[rustfmt::skip]
    const PRINT: [&str; 7] = [
        "nozzle_temperature_initial_layer", "nozzle_temperature", "activate_air_filtration",
        "activate_air_filtration_during_print", "activate_air_filtration_on_completion",
        "during_print_exhaust_fan_speed", "complete_print_exhaust_fan_speed",
    ];
    #[rustfmt::skip]
    const REGION: [&str; 4] = [
        "filament_ironing_flow", "filament_ironing_spacing",
        "filament_ironing_inset", "filament_ironing_speed",
    ];
    #[rustfmt::skip]
    const RETRACT: [&str; 16] = [
        "filament_retraction_length", "filament_z_hop", "filament_z_hop_types",
        "filament_retract_lift_above", "filament_retract_lift_below", "filament_retract_lift_enforce",
        "filament_retract_restart_extra", "filament_retraction_speed", "filament_deretraction_speed",
        "filament_retraction_minimum_travel", "filament_retract_when_changing_layer", "filament_wipe",
        "filament_wipe_distance", "filament_retract_before_wipe", "filament_long_retractions_when_cut",
        "filament_retraction_distances_when_cut",
    ];
    let declared = |fields: &[&str], declarations: &[&str]| {
        assert!(fields.iter().all(|field| declarations.contains(field)));
    };
    declared(&GCODE, &FilamentGCodeSourceOptions::DECLARATION_ORDER);
    declared(&PRINT, &FilamentPrintSourceOptions::DECLARATION_ORDER);
    declared(&REGION, &FilamentRegionSourceOptions::DECLARATION_ORDER);
    declared(&RETRACT, &FilamentRetractOverrideOptions::DECLARATION_ORDER);
    assert_eq!(GCODE.len() + PRINT.len() + REGION.len() + RETRACT.len(), 37);
    let unique = GCODE
        .into_iter()
        .chain(PRINT)
        .chain(REGION)
        .chain(RETRACT)
        .collect::<BTreeSet<_>>();
    assert_eq!(unique.len(), 37);
}

#[test]
fn omitted_root_default_expands_before_reordered_child_mapping() {
    let options = merged(
        [
            br#"{"type":"filament","name":"root","filament_extruder_variant":["A","B"]}"# as &[u8],
            br#"{"type":"filament","name":"child","inherits":"root","filament_extruder_variant":["B","A"],"filament_flow_ratio":[1.2,0.8]}"#,
        ],
        "child",
    );
    assert_eq!(options.gcode.filament_extruder_variant.0, ["A", "B"]);
    assert_eq!(
        floats(&options.gcode.filament_flow_ratio),
        [Some(0.8), Some(1.2)]
    );
}

#[test]
fn root_nil_reset_differs_from_retract_override_preservation() {
    let options = merged(
        [br#"{"type":"filament","name":"root","filament_extruder_variant":["A","B"],"filament_flow_ratio":["nil","nil"],"filament_retraction_length":["nil","nil"]}"# as &[u8]],
        "root",
    );
    assert_eq!(
        floats(&options.gcode.filament_flow_ratio),
        [Some(1.0), Some(1.0)]
    );
    assert_eq!(
        floats(&options.retract_overrides.filament_retraction_length),
        [None, None]
    );
}

#[test]
fn root_grows_truncates_resets_empty_vectors_and_clears_at_zero() {
    let options = merged(
        [br#"{"type":"filament","name":"root","filament_extruder_variant":["A","B","C"],"filament_flow_ratio":[0.8,0.9],"filament_flush_volumetric_speed":[],"filament_max_volumetric_speed":[],"nozzle_temperature_initial_layer":[],"nozzle_temperature":[210,220,230,240],"activate_air_filtration":[]}"# as &[u8]],
        "root",
    );
    let flow = floats(&options.gcode.filament_flow_ratio);
    assert_eq!(flow, [Some(0.8), Some(0.9), Some(0.8)]);
    assert_eq!(
        floats(&options.gcode.filament_flush_volumetric_speed),
        [Some(0.0); 3]
    );
    assert_eq!(
        options.gcode.filament_max_volumetric_speed.0,
        [OrcaFloat(2.0); 3]
    );
    assert_eq!(
        options.print.nozzle_temperature.0,
        [OrcaInt(210), OrcaInt(220), OrcaInt(230)]
    );
    let initial_nozzle = options.print.nozzle_temperature_initial_layer.0;
    assert_eq!(initial_nozzle, [OrcaInt(200); 3]);
    assert_eq!(
        options.print.activate_air_filtration.0,
        [OrcaBool(false); 3]
    );
    let zero = merged(
        [br#"{"type":"filament","name":"zero","filament_extruder_variant":[],"filament_flow_ratio":[0.8],"nozzle_temperature":[210],"filament_retraction_length":[0.9]}"# as &[u8]],
        "zero",
    );
    assert!(zero.gcode.filament_extruder_variant.0.is_empty());
    assert!(zero.gcode.filament_flow_ratio.is_empty());
    assert!(zero.print.nozzle_temperature.0.is_empty());
    assert!(zero.retract_overrides.filament_retraction_length.is_empty());
}

#[test]
fn root_empty_string_and_retract_vectors_are_keyed_errors() {
    for (key, value) in [
        ("volumetric_speed_coefficients", "[]"),
        ("filament_retraction_length", "[]"),
    ] {
        let input = format!(
            r#"{{"type":"filament","name":"root","filament_extruder_variant":["A"],"{key}":{value}}}"#
        );
        let parsed = fragments([input.as_bytes()]);
        let frozen = parsed.clone();
        assert_invalid(merge_options(&parsed, "root"), key);
        assert_eq!(parsed, frozen);
    }
}

#[test]
fn child_grows_first_value_and_truncates_before_reordered_mapping() {
    let options = merged(
        [
            br#"{"type":"filament","name":"root","filament_extruder_variant":["A","B","C"],"filament_flow_ratio":[0.8,0.9,1.0],"nozzle_temperature":[200,210,215]}"# as &[u8],
            br#"{"type":"filament","name":"child","inherits":"root","filament_extruder_variant":["B","C","A"],"filament_flow_ratio":[1.2,1.3],"nozzle_temperature":[220,230,240,250]}"#,
        ],
        "child",
    );
    let flow = floats(&options.gcode.filament_flow_ratio);
    assert_eq!(flow, [Some(1.2), Some(1.2), Some(1.3)]);
    assert_eq!(
        options.print.nozzle_temperature.0,
        [OrcaInt(240), OrcaInt(220), OrcaInt(230)]
    );
}

#[test]
fn every_representative_positive_child_empty_stays_a_keyed_error() {
    for key in [
        "filament_flow_ratio",
        "filament_max_volumetric_speed",
        "nozzle_temperature",
        "activate_air_filtration",
        "volumetric_speed_coefficients",
        "filament_retraction_length",
    ] {
        let child = format!(r#"{{"type":"filament","name":"child","inherits":"root","{key}":[]}}"#);
        let parsed = fragments([
            br#"{"type":"filament","name":"root","filament_extruder_variant":["A"]}"# as &[u8],
            child.as_bytes(),
        ]);
        let frozen = parsed.clone();
        assert_invalid(merge_options(&parsed, "child"), key);
        assert_eq!(parsed, frozen);
    }
}

#[test]
fn zero_root_fallback_then_nil_inheritance_retains_omitted_family_data() {
    let options = merged(
        [
            br#"{"type":"filament","name":"root","filament_extruder_variant":[],"filament_flow_ratio":[0.8],"filament_flush_temp":[5]}"# as &[u8],
            br#"{"type":"filament","name":"child","inherits":"root","filament_flow_ratio":[1.2],"filament_flush_temp":[7]}"#,
            br#"{"type":"filament","name":"grand","inherits":"child","filament_flow_ratio":["nil"]}"#,
        ],
        "grand",
    );
    assert!(options.gcode.filament_extruder_variant.0.is_empty());
    assert_eq!(floats(&options.gcode.filament_flow_ratio), [Some(1.2)]);
    assert_eq!(
        options.gcode.filament_flush_temp,
        [Nullable::Value(OrcaInt(7))]
    );
}

#[test]
fn explicit_empty_child_identity_can_short_circuit_or_report_missing_slot() {
    let equal = merged(
        [
            br#"{"type":"filament","name":"root","filament_extruder_variant":[],"filament_flow_ratio":[0.8]}"# as &[u8],
            br#"{"type":"filament","name":"equal","inherits":"root","filament_extruder_variant":[],"filament_flow_ratio":[1.2]}"#,
        ],
        "equal",
    );
    assert!(equal.gcode.filament_flow_ratio.is_empty());
    let parsed = fragments([
        br#"{"type":"filament","name":"root","filament_extruder_variant":[]}"# as &[u8],
        br#"{"type":"filament","name":"child","inherits":"root","filament_flow_ratio":[1.2]}"#,
        br#"{"type":"filament","name":"bad","inherits":"child","filament_extruder_variant":[],"filament_flow_ratio":[1.3]}"#,
    ]);
    assert_invalid(merge_options(&parsed, "bad"), "filament_flow_ratio");
}

#[test]
fn mapping_handles_omission_duplicates_unmatched_child_only_and_later_child() {
    let omitted = merged(
        [
            br#"{"type":"filament","name":"root","filament_extruder_variant":["A","B"],"filament_flow_ratio":[0.8,0.9]}"# as &[u8],
            br#"{"type":"filament","name":"omitted","inherits":"root","filament_type":["PETG"]}"#,
        ],
        "omitted",
    );
    assert_eq!(
        floats(&omitted.gcode.filament_flow_ratio),
        [Some(0.8), Some(0.9)]
    );
    let root = br#"{"type":"filament","name":"root","filament_extruder_variant":["A","B","C"],"filament_flow_ratio":[1,1,1]}"# as &[u8];
    let child = br#"{"type":"filament","name":"child","inherits":"root","filament_extruder_variant":["B","A","A","X"],"filament_flow_ratio":[2,3,4,9]}"# as &[u8];
    let first = merged([root, child], "child");
    assert_eq!(
        floats(&first.gcode.filament_flow_ratio),
        [Some(3.0), Some(2.0), Some(1.0)]
    );
    let later = merged(
        [
            root,
            child,
            br#"{"type":"filament","name":"later","inherits":"child","filament_extruder_variant":["C","A","B"],"filament_flow_ratio":[6,4,5]}"#,
        ],
        "later",
    );
    assert_eq!(later.gcode.filament_extruder_variant.0, ["A", "B", "C"]);
    assert_eq!(
        floats(&later.gcode.filament_flow_ratio),
        [Some(4.0), Some(5.0), Some(6.0)]
    );
}

#[test]
fn all_four_owners_map_concrete_and_nil_slots_in_root_order() {
    let options = merged(
        [
            br#"{"type":"filament","name":"root","filament_extruder_variant":["A","B"],"filament_max_volumetric_speed":[10,20],"nozzle_temperature":[200,210],"filament_ironing_flow":[10,20],"filament_retraction_length":[0.8,0.9]}"# as &[u8],
            br#"{"type":"filament","name":"child","inherits":"root","filament_extruder_variant":["B","A"],"filament_max_volumetric_speed":[30,40],"nozzle_temperature":[220,230],"filament_ironing_flow":["nil",25],"filament_retraction_length":["nil",1.1]}"#,
        ],
        "child",
    );
    assert_eq!(
        options.gcode.filament_max_volumetric_speed.0,
        [OrcaFloat(40.0), OrcaFloat(30.0)]
    );
    assert_eq!(
        options.print.nozzle_temperature.0,
        [OrcaInt(230), OrcaInt(220)]
    );
    assert_eq!(
        percents(&options.region.filament_ironing_flow),
        [Some(25.0), Some(20.0)]
    );
    assert_eq!(
        floats(&options.retract_overrides.filament_retraction_length),
        [Some(1.1), Some(0.9)]
    );
}

#[test]
fn equality_short_circuit_is_type_directed_and_epsilon_is_strict() {
    let options = merged(
        [
            br#"{"type":"filament","name":"root","filament_extruder_variant":["A","B"],"filament_flow_ratio":[1,2],"filament_flush_temp":[1,2],"filament_ironing_flow":[10,20],"filament_ironing_spacing":[0,1],"filament_max_volumetric_speed":[3,4]}"# as &[u8],
            br#"{"type":"filament","name":"child","inherits":"root","filament_extruder_variant":["B","A"],"filament_flow_ratio":[1.00005,2.00005],"filament_flush_temp":[1,2],"filament_ironing_flow":[10.00005,20.00005],"filament_ironing_spacing":[0.0001,1],"filament_max_volumetric_speed":[3.00005,4.00005]}"#,
        ],
        "child",
    );
    assert_eq!(
        floats(&options.gcode.filament_flow_ratio),
        [Some(1.0), Some(2.0)]
    );
    assert_eq!(
        options.gcode.filament_flush_temp,
        [Nullable::Value(OrcaInt(1)), Nullable::Value(OrcaInt(2))]
    );
    assert_eq!(
        percents(&options.region.filament_ironing_flow),
        [Some(10.0), Some(20.0)]
    );
    assert_eq!(
        floats(&options.region.filament_ironing_spacing),
        [Some(1.0), Some(0.0001)]
    );
    assert_eq!(
        options.gcode.filament_max_volumetric_speed.0,
        [OrcaFloat(4.00005), OrcaFloat(3.00005)]
    );
}

#[test]
fn non_variant_overlay_and_profile_inputs_remain_atomic() {
    let parsed = fragments([
        br#"{"type":"filament","name":"root","filament_extruder_variant":["A","B"],"filament_type":["PLA","PETG"]}"# as &[u8],
        br#"{"type":"filament","name":"child","inherits":"root","filament_extruder_variant":["B","A"],"filament_type":["ABS"]}"#,
    ]);
    let frozen = parsed.clone();
    let options = merge_options(&parsed, "child").unwrap();
    assert_eq!(options.gcode.filament_type.0, ["ABS"]);
    assert_eq!(options.gcode.filament_extruder_variant.0, ["A", "B"]);
    assert_eq!(parsed, frozen);
}

#[test]
fn malformed_family_shapes_are_keyed_invalid_input_without_mutation() {
    #[rustfmt::skip]
    const CASES: [(&str, &[u8]); 2] = [
        ("filament_flow_ratio", br#"{"type":"filament","name":"bad","filament_flow_ratio":1}"# as &[u8]),
        ("filament_extruder_variant", br#"{"type":"filament","name":"bad","filament_extruder_variant":[1]}"#),
    ];
    for (key, input) in CASES {
        let frozen = input.to_vec();
        assert_invalid(ProfileFragment::from_json_bytes(input), key);
        assert_eq!(input, frozen);
    }
}

#[test]
#[rustfmt::skip]
fn variant_source_length_mismatch_whole_replaces_child() {
    let mapping = [Some(usize::MAX)];
    let child = vec![Nullable::Value(OrcaFloat(1.2))];
    let compared = Cell::new(false);
    let apply = |source: &mut Vec<Nullable<OrcaFloat>>| -> Result<(), SliceError> {
        crate::options::option_group::apply_variant_slots(
            source, &child, &mapping, "filament_flow_ratio",
            (
                |source, child| source == child || { compared.set(true); assert!(source.is_empty()); false },
                |value| matches!(value, Nullable::Value(_)),
            ),
        )
    };
    let mut source = child.clone();
    apply(&mut source).unwrap();
    assert_eq!(source, child);
    source.clear();
    apply(&mut source).unwrap();
    assert!(compared.get());
    assert_eq!(source, child);
}
