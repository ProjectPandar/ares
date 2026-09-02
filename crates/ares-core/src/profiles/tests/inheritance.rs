use crate::{MergedProfile, Nullable, OrcaFloat, ProfileKind, merge_profile_fragments};

use super::{assert_invalid, fragment, fragments};

#[test]
fn exhaustive_tagged_merge_exposes_each_concrete_owner() {
    let fragments = fragments([
        br#"{"type":"machine","name":"shared","printer_model":"Ares M"}"# as &[u8],
        br#"{"type":"process","name":"shared","layer_height":0.16}"#,
        br#"{"type":"filament","name":"shared","filament_diameter":[1.73]}"#,
    ]);

    match merge_profile_fragments(&fragments, ProfileKind::Machine, "shared").unwrap() {
        MergedProfile::Machine { metadata, options } => {
            assert_eq!(metadata.name(), "shared");
            assert_eq!(options.remaining.printer_model.0, "Ares M");
        }
        other => panic!("expected machine result, got {other:?}"),
    }
    match merge_profile_fragments(&fragments, ProfileKind::Process, "shared").unwrap() {
        MergedProfile::Process { metadata, options } => {
            assert_eq!(metadata.name(), "shared");
            assert_eq!(options.object.layer_height.0, 0.16);
        }
        other => panic!("expected process result, got {other:?}"),
    }
    match merge_profile_fragments(&fragments, ProfileKind::Filament, "shared").unwrap() {
        MergedProfile::Filament { metadata, options } => {
            assert_eq!(metadata.name(), "shared");
            assert_eq!(options.gcode.filament_diameter.0[0].0, 1.73);
        }
        other => panic!("expected filament result, got {other:?}"),
    }
}

#[test]
fn grandparent_parent_child_overlay_is_parent_first_and_resolves_once() {
    let fragments = fragments([
        br#"{"type":"process","name":"grand","layer_height":0.3,"wall_loops":7}"# as &[u8],
        br#"{"type":"process","name":"parent","inherits":"grand","layer_height":0.25}"#,
        br#"{"type":"process","name":"child","inherits":"parent","wall_loops":2}"#,
    ]);

    let MergedProfile::Process { metadata, options } =
        merge_profile_fragments(&fragments, ProfileKind::Process, "child").unwrap()
    else {
        panic!("process target returned the wrong tagged owner")
    };

    assert_eq!(metadata.name(), "child");
    assert_eq!(metadata.inherits(), Some("parent"));
    assert_eq!(options.object.layer_height.0, 0.25);
    assert_eq!(options.region.wall_loops.0, 2);
}

#[test]
fn child_explicit_fixed_default_overrides_a_different_parent_value() {
    let parent =
        fragment(br#"{"type":"process","name":"parent","layer_height":0.31,"wall_loops":8}"#);
    let omitted =
        fragment(br#"{"type":"process","name":"child","inherits":"parent","wall_loops":2}"#);
    let explicit_default = fragment(
        br#"{"type":"process","name":"child","inherits":"parent","layer_height":0.2,"wall_loops":2}"#,
    );

    assert_ne!(omitted, explicit_default);

    let MergedProfile::Process {
        options: inherited, ..
    } = merge_profile_fragments(&[parent.clone(), omitted], ProfileKind::Process, "child").unwrap()
    else {
        unreachable!()
    };
    let MergedProfile::Process {
        options: overridden,
        ..
    } = merge_profile_fragments(&[parent, explicit_default], ProfileKind::Process, "child")
        .unwrap()
    else {
        unreachable!()
    };

    assert_eq!(inherited.object.layer_height.0, 0.31);
    assert_eq!(overridden.object.layer_height.0, 0.2);
    assert_eq!(inherited.region.wall_loops.0, 2);
    assert_eq!(overridden.region.wall_loops.0, 2);
}

#[test]
fn present_nullable_vector_normalizes_and_inherits_slot_zero() {
    let fragments = fragments([
        br#"{"type":"filament","name":"parent","filament_flow_ratio":[0.9,"nil",1.1]}"# as &[u8],
        br#"{"type":"filament","name":"child","inherits":"parent","filament_flow_ratio":["nil",1.2]}"#,
    ]);

    let MergedProfile::Filament { options, .. } =
        merge_profile_fragments(&fragments, ProfileKind::Filament, "child").unwrap()
    else {
        unreachable!()
    };
    assert_eq!(
        options.gcode.filament_flow_ratio,
        vec![Nullable::Value(OrcaFloat(0.9))]
    );
}

#[test]
fn omitted_nullable_vector_retains_normalized_parent_slot_zero() {
    let fragments = fragments([
        br#"{"type":"filament","name":"parent","filament_flow_ratio":[0.9,"nil"]}"# as &[u8],
        br#"{"type":"filament","name":"child","inherits":"parent","filament_type":["PLA"]}"#,
    ]);

    let MergedProfile::Filament { options, .. } =
        merge_profile_fragments(&fragments, ProfileKind::Filament, "child").unwrap()
    else {
        unreachable!()
    };
    assert_eq!(
        options.gcode.filament_flow_ratio,
        vec![Nullable::Value(OrcaFloat(0.9))]
    );
}

#[test]
fn compatibility_metadata_omits_overrides_and_explicitly_clears_by_presence() {
    let fragments = fragments([
        br#"{"type":"filament","name":"parent","compatible_printers":["M1","M2"],"compatible_printers_condition":"parent-machine","compatible_prints":["P1"],"compatible_prints_condition":"parent-process"}"# as &[u8],
        br#"{"type":"filament","name":"omits","inherits":"parent"}"#,
        br#"{"type":"filament","name":"overrides","inherits":"parent","compatible_printers":["M3"],"compatible_prints_condition":"child-process"}"#,
        br#"{"type":"filament","name":"clears","inherits":"parent","compatible_printers":[],"compatible_printers_condition":"","compatible_prints":[],"compatible_prints_condition":""}"#,
    ]);

    let MergedProfile::Filament {
        metadata: omitted, ..
    } = merge_profile_fragments(&fragments, ProfileKind::Filament, "omits").unwrap()
    else {
        unreachable!()
    };
    assert_eq!(omitted.compatible_printers().unwrap(), ["M1", "M2"]);
    assert_eq!(
        omitted.compatible_printers_condition(),
        Some("parent-machine")
    );
    assert_eq!(omitted.compatible_prints().unwrap(), ["P1"]);
    assert_eq!(
        omitted.compatible_prints_condition(),
        Some("parent-process")
    );

    let MergedProfile::Filament {
        metadata: overridden,
        ..
    } = merge_profile_fragments(&fragments, ProfileKind::Filament, "overrides").unwrap()
    else {
        unreachable!()
    };
    assert_eq!(overridden.compatible_printers().unwrap(), ["M3"]);
    assert_eq!(
        overridden.compatible_printers_condition(),
        Some("parent-machine")
    );
    assert_eq!(overridden.compatible_prints().unwrap(), ["P1"]);
    assert_eq!(
        overridden.compatible_prints_condition(),
        Some("child-process")
    );

    let MergedProfile::Filament {
        metadata: cleared, ..
    } = merge_profile_fragments(&fragments, ProfileKind::Filament, "clears").unwrap()
    else {
        unreachable!()
    };
    assert!(
        cleared
            .compatible_printers()
            .is_some_and(<[String]>::is_empty)
    );
    assert_eq!(cleared.compatible_printers_condition(), Some(""));
    assert!(
        cleared
            .compatible_prints()
            .is_some_and(<[String]>::is_empty)
    );
    assert_eq!(cleared.compatible_prints_condition(), Some(""));
}

#[test]
fn process_compatibility_metadata_uses_the_same_presence_overlay() {
    let fragments = fragments([
        br#"{"type":"process","name":"parent","compatible_printers":["M1","M2"],"compatible_printers_condition":"parent-machine"}"# as &[u8],
        br#"{"type":"process","name":"omits","inherits":"parent"}"#,
        br#"{"type":"process","name":"overrides","inherits":"parent","compatible_printers":["M3"]}"#,
        br#"{"type":"process","name":"clears","inherits":"parent","compatible_printers":[],"compatible_printers_condition":""}"#,
    ]);

    let MergedProfile::Process {
        metadata: omitted, ..
    } = merge_profile_fragments(&fragments, ProfileKind::Process, "omits").unwrap()
    else {
        unreachable!()
    };
    assert_eq!(omitted.compatible_printers().unwrap(), ["M1", "M2"]);
    assert_eq!(
        omitted.compatible_printers_condition(),
        Some("parent-machine")
    );

    let MergedProfile::Process {
        metadata: overridden,
        ..
    } = merge_profile_fragments(&fragments, ProfileKind::Process, "overrides").unwrap()
    else {
        unreachable!()
    };
    assert_eq!(overridden.compatible_printers().unwrap(), ["M3"]);
    assert_eq!(
        overridden.compatible_printers_condition(),
        Some("parent-machine")
    );

    let MergedProfile::Process {
        metadata: cleared, ..
    } = merge_profile_fragments(&fragments, ProfileKind::Process, "clears").unwrap()
    else {
        unreachable!()
    };
    assert!(
        cleared
            .compatible_printers()
            .is_some_and(<[String]>::is_empty)
    );
    assert_eq!(cleared.compatible_printers_condition(), Some(""));
}

#[test]
fn selected_loader_identity_does_not_inherit_but_filament_id_follows_parent() {
    let fragments = fragments([
        br#"{"type":"filament","name":"root","from":"system","version":"root-version","setting_id":"ROOT-SETTING","instantiation":"root-instantiation","description":"root description","url":"https://root.invalid","renamed_from":"root-legacy","filament_id":"ROOT-ID"}"# as &[u8],
        br#"{"type":"filament","name":"parent","inherits":"root","filament_id":"PARENT-ID"}"#,
        br#"{"type":"filament","name":"empty-child","inherits":"parent","filament_id":"EMPTY-CHILD-ID"}"#,
        br#"{"type":"filament","name":"child","inherits":"parent","from":"user","version":"child-version","setting_id":"CHILD-SETTING","instantiation":"child-instantiation","description":"child description","url":"https://child.invalid","renamed_from":"child-legacy","filament_id":"CHILD-ID"}"#,
        br#"{"type":"filament","name":"standalone","filament_id":"OWN-ID"}"#,
    ]);

    let MergedProfile::Filament {
        metadata: empty_child,
        ..
    } = merge_profile_fragments(&fragments, ProfileKind::Filament, "empty-child").unwrap()
    else {
        unreachable!()
    };
    assert_eq!(empty_child.from(), None);
    assert_eq!(empty_child.version(), None);
    assert_eq!(empty_child.setting_id(), None);
    assert_eq!(empty_child.instantiation(), None);
    assert_eq!(empty_child.description(), None);
    assert_eq!(empty_child.url(), None);
    assert_eq!(empty_child.renamed_from(), None);
    assert_eq!(empty_child.filament_id(), Some("ROOT-ID"));

    let MergedProfile::Filament {
        metadata: child, ..
    } = merge_profile_fragments(&fragments, ProfileKind::Filament, "child").unwrap()
    else {
        unreachable!()
    };
    assert_eq!(child.name(), "child");
    assert_eq!(child.from(), Some("user"));
    assert_eq!(child.version(), Some("child-version"));
    assert_eq!(child.setting_id(), Some("CHILD-SETTING"));
    assert_eq!(child.instantiation(), Some("child-instantiation"));
    assert_eq!(child.description(), Some("child description"));
    assert_eq!(child.url(), Some("https://child.invalid"));
    assert_eq!(child.renamed_from(), Some("child-legacy"));
    assert_eq!(child.filament_id(), Some("ROOT-ID"));

    let MergedProfile::Filament { metadata: root, .. } =
        merge_profile_fragments(&fragments, ProfileKind::Filament, "standalone").unwrap()
    else {
        unreachable!()
    };
    assert_eq!(root.filament_id(), Some("OWN-ID"));
}

#[test]
fn fragment_input_order_does_not_affect_the_merged_result() {
    let ordered = fragments([
        br#"{"type":"machine","name":"root","printer_model":"root","thumbnails":"32x48"}"#
            as &[u8],
        br#"{"type":"machine","name":"parent","inherits":"root","printer_model":"parent"}"#,
        br#"{"type":"machine","name":"child","inherits":"parent","thumbnails_format":"QOI"}"#,
    ]);
    let reversed = ordered.iter().cloned().rev().collect::<Vec<_>>();

    let ordered = merge_profile_fragments(&ordered, ProfileKind::Machine, "child").unwrap();
    let reversed = merge_profile_fragments(&reversed, ProfileKind::Machine, "child").unwrap();
    assert_eq!(ordered, reversed);

    let MergedProfile::Machine { options, .. } = ordered else {
        unreachable!()
    };
    assert_eq!(options.remaining.thumbnails.as_str(), "32x48");
    assert_eq!(options.remaining.thumbnails_format.as_str(), "QOI");
}

#[test]
fn duplicate_missing_cross_kind_self_and_long_cycle_errors_are_atomic() {
    let cases = [
        fragments([
            br#"{"type":"process","name":"same"}"# as &[u8],
            br#"{"type":"process","name":"same"}"#,
        ]),
        fragments([
            br#"{"type":"process","name":"child","inherits":"missing"}"# as &[u8],
            br#"{"type":"machine","name":"m"}"#,
        ]),
        fragments([
            br#"{"type":"process","name":"child","inherits":"parent"}"# as &[u8],
            br#"{"type":"machine","name":"parent"}"#,
        ]),
        fragments([
            br#"{"type":"process","name":"self","inherits":"self"}"# as &[u8],
            br#"{"type":"machine","name":"m"}"#,
        ]),
        fragments([
            br#"{"type":"process","name":"a","inherits":"b"}"# as &[u8],
            br#"{"type":"process","name":"b","inherits":"c"}"#,
            br#"{"type":"process","name":"c","inherits":"a"}"#,
        ]),
    ];

    for fragments in cases {
        let frozen = fragments.clone();
        let target = fragments[0].name().to_owned();
        assert_invalid(
            merge_profile_fragments(&fragments, ProfileKind::Process, &target),
            "profile",
        );
        assert_eq!(fragments, frozen);
    }
}

#[test]
fn missing_target_does_not_return_a_partial_result() {
    let fragment = fragment(br#"{"type":"process","name":"only","layer_height":0.2}"#);
    let fragments = vec![fragment];
    assert_invalid(
        merge_profile_fragments(&fragments, ProfileKind::Process, "absent"),
        "profile",
    );
}
