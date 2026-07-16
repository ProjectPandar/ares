use crate::{ProfileFragment, ProfileKind, ProfileSelection, merge_profile_fragments};

use super::{assert_invalid, fragment, fragments};

#[test]
fn malformed_non_object_and_trailing_json_are_rejected_atomically() {
    for input in [
        br#"{"type":"process"# as &[u8],
        br#"[]"#,
        br#"{"type":"process","name":"fine"} trailing"#,
    ] {
        let frozen = input.to_vec();
        assert_invalid(ProfileFragment::from_json_bytes(input), "profile");
        assert_eq!(input, frozen);
    }
}

#[test]
fn required_metadata_rejects_missing_duplicate_wrong_type_unsupported_and_empty() {
    for input in [
        br#"{"name":"fine"}"# as &[u8],
        br#"{"type":"process"}"#,
        br#"{"type":"process","type":"machine","name":"fine"}"#,
        br#"{"type":"process","name":"fine","name":"other"}"#,
        br#"{"type":1,"name":"fine"}"#,
        br#"{"type":"process","name":1}"#,
        br#"{"type":"resin","name":"fine"}"#,
        br#"{"type":"process","name":""}"#,
    ] {
        assert_invalid(ProfileFragment::from_json_bytes(input), "profile");
    }
}

#[test]
fn optional_metadata_rejects_duplicates_and_non_strings() {
    let fields = [
        "inherits",
        "from",
        "version",
        "setting_id",
        "instantiation",
        "description",
        "url",
        "renamed_from",
        "filament_id",
    ];
    for field in fields {
        let duplicate = format!(r#"{{"type":"filament","name":"f","{field}":"a","{field}":"b"}}"#);
        let wrong_type = format!(r#"{{"type":"filament","name":"f","{field}":["bad"]}}"#);
        assert_invalid(
            ProfileFragment::from_json_bytes(duplicate.as_bytes()),
            "profile",
        );
        assert_invalid(
            ProfileFragment::from_json_bytes(wrong_type.as_bytes()),
            "profile",
        );
    }
}

#[test]
fn unknown_misplaced_duplicate_and_malformed_options_are_rejected() {
    for input in [
        br#"{"type":"process","name":"p","future_option":1}"# as &[u8],
        br#"{"type":"process","name":"p","filament_diameter":[1.75]}"#,
        br#"{"type":"filament","name":"f","printer_model":"X1"}"#,
        br#"{"type":"machine","name":"m","layer_height":0.2}"#,
        br#"{"type":"process","name":"p","layer_height":0.2,"layer_height":0.1}"#,
        br#"{"type":"process","name":"p","layer_height":{"bad":true}}"#,
    ] {
        assert_invalid(ProfileFragment::from_json_bytes(input), "option");
    }

    let malformed_thumbnails =
        fragment(br#"{"type":"machine","name":"malformed-thumbnails","thumbnails":"broken"}"#);
    assert_invalid(
        merge_profile_fragments(
            &[malformed_thumbnails],
            ProfileKind::Machine,
            "malformed-thumbnails",
        ),
        "profile option thumbnails",
    );
}

#[test]
fn compatibility_keys_are_kind_owned_and_machine_accepts_none() {
    let process = fragment(
        br#"{"type":"process","name":"p","compatible_printers":["M"],"compatible_printers_condition":"ok"}"#,
    );
    let filament = fragment(
        br#"{"type":"filament","name":"f","compatible_printers":["M"],"compatible_printers_condition":"m","compatible_prints":["P"],"compatible_prints_condition":"p"}"#,
    );
    assert_eq!(process.kind(), ProfileKind::Process);
    assert_eq!(filament.kind(), ProfileKind::Filament);

    for field in [
        r#""compatible_printers":["M"]"#,
        r#""compatible_printers_condition":"m""#,
        r#""compatible_prints":["P"]"#,
        r#""compatible_prints_condition":"p""#,
    ] {
        let input = format!(r#"{{"type":"machine","name":"m",{field}}}"#);
        assert_invalid(
            ProfileFragment::from_json_bytes(input.as_bytes()),
            "profile",
        );
    }

    for field in [
        r#""compatible_prints":["P"]"#,
        r#""compatible_prints_condition":"p""#,
    ] {
        let input = format!(r#"{{"type":"process","name":"p",{field}}}"#);
        assert_invalid(
            ProfileFragment::from_json_bytes(input.as_bytes()),
            "profile",
        );
    }

    for kind in ["process", "machine"] {
        let input = format!(r#"{{"type":"{kind}","name":"wrong-id","filament_id":"F-ID"}}"#);
        assert_invalid(
            ProfileFragment::from_json_bytes(input.as_bytes()),
            "profile",
        );
    }
}

#[test]
fn compatibility_values_reject_wrong_shape_and_duplicates() {
    for input in [
        br#"{"type":"process","name":"p","compatible_printers":"M"}"# as &[u8],
        br#"{"type":"process","name":"p","compatible_printers_condition":["bad"]}"#,
        br#"{"type":"filament","name":"f","compatible_prints":[1]}"#,
        br#"{"type":"filament","name":"f","compatible_prints_condition":false}"#,
        br#"{"type":"process","name":"p","compatible_printers":[],"compatible_printers":[]}"#,
        br#"{"type":"process","name":"p","compatible_printers_condition":"a","compatible_printers_condition":"b"}"#,
        br#"{"type":"filament","name":"f","compatible_prints":[],"compatible_prints":[]}"#,
        br#"{"type":"filament","name":"f","compatible_prints_condition":"a","compatible_prints_condition":"b"}"#,
    ] {
        assert_invalid(ProfileFragment::from_json_bytes(input), "profile");
    }
}

#[test]
fn empty_selection_and_missing_selected_profile_errors_are_stable_and_atomic() {
    for result in [
        ProfileSelection::new("", "m", ["f"]),
        ProfileSelection::new("p", "", ["f"]),
        ProfileSelection::new("p", "m", Vec::<&str>::new()),
        ProfileSelection::new("p", "m", [""]),
    ] {
        assert_invalid(result, "selection");
    }

    let fragments = fragments([
        br#"{"type":"process","name":"p"}"# as &[u8],
        br#"{"type":"machine","name":"m"}"#,
        br#"{"type":"filament","name":"f"}"#,
    ]);
    let frozen = fragments.clone();
    assert_invalid(
        merge_profile_fragments(&fragments, ProfileKind::Process, "missing"),
        "profile",
    );
    assert_eq!(fragments, frozen);
}
