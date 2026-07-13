use super::super::super::{OrcaInt, RegionOptionOverrides};
use super::{inventory, region_rows};

#[test]
fn sparse_overrides_distinguish_absence_from_default_valued_presence() {
    let mut overrides = RegionOptionOverrides::default();
    assert!(overrides.present_keys().is_empty());
    assert_eq!(overrides.bottom_shell_layers, None);

    assert!(overrides
        .deserialize_known_field("bottom_shell_layers", "3")
        .unwrap());
    assert_eq!(overrides.bottom_shell_layers, Some(OrcaInt(3)));
    assert_eq!(overrides.present_keys(), ["bottom_shell_layers"]);

    assert!(overrides
        .deserialize_known_field("extruder", "0")
        .unwrap());
    assert_eq!(overrides.extruder, Some(OrcaInt(0)));
    assert_eq!(
        overrides.present_keys(),
        ["bottom_shell_layers", "extruder"]
    );
}

#[test]
fn every_region_key_dispatches_into_only_its_concrete_slot() {
    let rows = inventory();
    for row in region_rows(&rows) {
        let mut overrides = RegionOptionOverrides::default();
        assert!(
            overrides
                .deserialize_known_field(&row.key, &row.default_serialized)
                .unwrap(),
            "{}",
            row.key
        );
        assert_eq!(overrides.present_keys(), [row.key.as_str()], "{}", row.key);
    }

    let mut overrides = RegionOptionOverrides::default();
    assert!(!overrides
        .deserialize_known_field("not_a_region_option", "1")
        .unwrap());
    assert!(overrides.present_keys().is_empty());
}

#[test]
fn every_region_codec_reports_its_key_for_malformed_metadata() {
    let rows = inventory();
    for row in region_rows(&rows) {
        let malformed = match row.option_type.as_str() {
            "coBool" => "2",
            "coEnum" => "not-an-enum",
            "coFloat" | "coFloatOrPercent" | "coPercent" => "not-a-number",
            "coInt" => "not-an-integer",
            "coInts" => "1,,2",
            "coString" => "trailing\\",
            "coStrings" => "\"unterminated",
            other => panic!("unexpected region option type {other}"),
        };
        let error = RegionOptionOverrides::default()
            .deserialize_known_field(&row.key, malformed)
            .unwrap_err();
        assert!(error.to_string().contains(&row.key), "{error}");
    }
}
