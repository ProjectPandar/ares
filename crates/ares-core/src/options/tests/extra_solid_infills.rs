use crate::{SliceError, SliceOptions, options::ExtraSolidInfills};
use serde_json::json;

fn matches(pattern: &str, layer_index: usize) -> bool {
    let options: SliceOptions = serde_json::from_value(json!({
        "extra_solid_infills": pattern
    }))
    .unwrap();

    options
        .infill_options()
        .unwrap()
        .extra_solid_infills_matches_layer_for_tests(layer_index)
}

#[test]
fn empty_extra_solid_infills_schedule_matches_no_layers() {
    assert!(!matches("", 0));
    assert!(!matches("   ", 1));
}

#[test]
fn explicit_extra_solid_infills_list_uses_one_based_layers() {
    assert!(matches("1, 3, 5", 0));
    assert!(!matches("1, 3, 5", 1));
    assert!(matches("1, 3, 5", 2));
}

#[test]
fn repeating_extra_solid_infills_pattern_matches_every_nth_layer() {
    assert!(!matches("3", 0));
    assert!(!matches("3", 1));
    assert!(matches("3", 2));
    assert!(!matches("3", 3));
    assert!(!matches("3", 4));
    assert!(matches("3", 5));
}

#[test]
fn repeating_extra_solid_infills_pattern_supports_consecutive_count() {
    assert!(!matches("3#2", 0));
    assert!(!matches("3#2", 1));
    assert!(matches("3#2", 2));
    assert!(matches("3#2", 3));
    assert!(!matches("3#2", 4));
    assert!(matches("3#2", 5));
    assert!(matches("3#2", 6));
}

#[test]
fn hash_without_count_defaults_to_one_layer() {
    assert!(matches("3#", 2));
    assert!(!matches("3#", 3));
}

#[test]
fn comma_list_hash_entries_are_explicit_ranges_not_repeating_intervals() {
    assert!(matches("2#2,8", 1));
    assert!(matches("2#2,8", 2));
    assert!(!matches("2#2,8", 3));
    assert!(!matches("2#2,8", 5));
    assert!(matches("2#2,8", 7));
}

#[test]
fn whitespace_and_one_pair_of_outer_quotes_are_ignored() {
    assert!(matches("  \" 3#2 \" ", 2));
    assert!(matches("  ' 3#2 ' ", 3));
    assert!(!matches("  \" 3#2 \" ", 4));
}

#[test]
fn invalid_extra_solid_infills_tokens_are_rejected() {
    for pattern in [
        "abc",
        "0",
        "-1",
        "2#0",
        "2#abc",
        "1,,2",
        "#2",
        "2147483648",
        "1#2147483648",
    ] {
        let options: SliceOptions = serde_json::from_value(json!({
            "extra_solid_infills": pattern
        }))
        .unwrap();
        let err = options.infill_options().unwrap_err();
        assert_eq!(
            err,
            SliceError::InvalidInput("invalid extra_solid_infills pattern".to_owned())
        );
    }
}

#[test]
fn raw_parser_and_json_delegate_share_source_sized_boundaries() {
    for pattern in [
        "",
        " \t' 3#2 ' \n",
        "2147483647",
        "2147483646#1,2147483647",
    ] {
        let raw = ExtraSolidInfills::parse_raw(pattern).unwrap();
        let options: SliceOptions = serde_json::from_value(json!({
            "extra_solid_infills": pattern
        }))
        .unwrap();
        let json = options.infill_options().unwrap().extra_solid_infills;
        assert_eq!(raw, json);
    }

    for pattern in ["2147483648", "2147483647#2147483648"] {
        let raw = ExtraSolidInfills::parse_raw(pattern).unwrap_err();
        let options: SliceOptions = serde_json::from_value(json!({
            "extra_solid_infills": pattern
        }))
        .unwrap();
        assert_eq!(raw, options.infill_options().unwrap_err());
    }
}

#[test]
fn matching_near_source_bound_and_impossible_index_never_overflows() {
    let schedule = ExtraSolidInfills::parse_raw("2147483646#2147483647,1").unwrap();
    assert!(schedule.matches_layer(usize::try_from(i32::MAX).unwrap() - 2));
    assert!(!schedule.matches_layer(usize::MAX));
}
