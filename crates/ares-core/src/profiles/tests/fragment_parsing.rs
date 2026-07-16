use crate::{ProfileFragment, ProfileKind, merge_profile_fragments};

use super::{assert_invalid, fragment};

#[test]
fn metadata_first_options_first_and_type_last_are_order_independent() {
    let metadata_first =
        fragment(br#"{"type":"process","name":"fine","version":"2.4.2","layer_height":0.12}"#);
    let options_first =
        fragment(br#"{"layer_height":0.12,"version":"2.4.2","name":"fine","type":"process"}"#);
    let type_last =
        fragment(br#"{"name":"fine","layer_height":0.12,"version":"2.4.2","type":"process"}"#);

    assert_eq!(metadata_first, options_first);
    assert_eq!(options_first, type_last);
    assert_eq!(type_last.kind(), ProfileKind::Process);
    assert_eq!(type_last.name(), "fine");
    assert_eq!(type_last.version(), Some("2.4.2"));
}

#[test]
fn every_loader_metadata_field_is_typed_and_accessible() {
    let fragment = fragment(
        br#"{
            "type":"filament",
            "name":"PLA Basic",
            "from":"system",
            "version":"01.002.000.00",
            "setting_id":"GFSA00",
            "instantiation":"true",
            "description":"general purpose",
            "url":"https://example.invalid/pla",
            "renamed_from":"PLA Legacy;PLA Old",
            "filament_id":"GFA00",
            "filament_type":["PLA"]
        }"#,
    );

    assert_eq!(fragment.kind(), ProfileKind::Filament);
    assert_eq!(fragment.name(), "PLA Basic");
    assert_eq!(fragment.from(), Some("system"));
    assert_eq!(fragment.version(), Some("01.002.000.00"));
    assert_eq!(fragment.setting_id(), Some("GFSA00"));
    assert_eq!(fragment.instantiation(), Some("true"));
    assert_eq!(fragment.description(), Some("general purpose"));
    assert_eq!(fragment.url(), Some("https://example.invalid/pla"));
    assert_eq!(fragment.renamed_from(), Some("PLA Legacy;PLA Old"));
    assert_eq!(fragment.filament_id(), Some("GFA00"));
}

#[test]
fn exact_loader_strings_are_not_normalized_or_used_as_aliases() {
    let fragment = fragment(
        br#"{
            "type":"machine",
            "name":"printer",
            "version":" v2.4.2+vendor ",
            "instantiation":" FALSE ",
            "renamed_from":"renamed target",
            "printer_model":"X1"
        }"#,
    );

    assert_eq!(fragment.version(), Some(" v2.4.2+vendor "));
    assert_eq!(fragment.instantiation(), Some(" FALSE "));
    assert_eq!(fragment.renamed_from(), Some("renamed target"));
    assert_invalid(
        merge_profile_fragments(&[fragment], ProfileKind::Machine, "renamed target"),
        "profile",
    );
}

#[test]
fn all_three_profile_kinds_decode_without_payload_exposure() {
    let machine = fragment(br#"{"type":"machine","name":"m","printer_model":"M"}"#);
    let process = fragment(br#"{"type":"process","name":"p","layer_height":0.2}"#);
    let filament = fragment(br#"{"type":"filament","name":"f","filament_diameter":[1.75]}"#);

    assert_eq!(machine.kind(), ProfileKind::Machine);
    assert_eq!(process.kind(), ProfileKind::Process);
    assert_eq!(filament.kind(), ProfileKind::Filament);
}

#[test]
fn missing_and_empty_inherits_are_equivalent_roots() {
    let missing = fragment(br#"{"type":"process","name":"root","layer_height":0.2}"#);
    let empty = fragment(br#"{"type":"process","name":"root","inherits":"","layer_height":0.2}"#);

    assert_eq!(missing.inherits(), None);
    assert_eq!(empty.inherits(), None);
    assert_eq!(missing, empty);
}

#[test]
fn profile_input_bytes_are_borrowed_and_never_mutated() {
    let input = br#"{"name":"fine","layer_height":0.12,"type":"process"}"#.to_vec();
    let frozen = input.clone();

    let parsed = ProfileFragment::from_json_bytes(&input).unwrap();

    assert_eq!(input, frozen);
    assert_eq!(parsed.name(), "fine");
}
