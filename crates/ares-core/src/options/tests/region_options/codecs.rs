use super::super::super::{OrcaInt, RegionOptionOverrides};

#[test]
fn integer_vector_metadata_is_trimmed_comma_separated_and_non_null() {
    let mut overrides = RegionOptionOverrides::default();
    assert!(overrides
        .deserialize_known_field("print_extruder_id", " 1, -2 , 3 ")
        .unwrap());
    assert_eq!(
        overrides.print_extruder_id.unwrap().0,
        [OrcaInt(1), OrcaInt(-2), OrcaInt(3)]
    );

    let mut empty = RegionOptionOverrides::default();
    empty
        .deserialize_known_field("print_extruder_id", "")
        .unwrap();
    assert!(empty.print_extruder_id.unwrap().0.is_empty());

    for malformed in [" ", "1,,2", "1,null", "1.5"] {
        let error = RegionOptionOverrides::default()
            .deserialize_known_field("print_extruder_id", malformed)
            .unwrap_err();
        assert!(error.to_string().contains("print_extruder_id"), "{error}");
    }
}

#[test]
fn scalar_string_metadata_uses_orca_c_style_unescape() {
    let mut overrides = RegionOptionOverrides::default();
    assert!(overrides
        .deserialize_known_field(
            "solid_infill_rotate_template",
            r"carriage\rreturn\nline\\slash\q",
        )
        .unwrap());
    assert_eq!(
        overrides.solid_infill_rotate_template.unwrap().0,
        "carriage\rreturn\nline\\slashq"
    );

    let error = RegionOptionOverrides::default()
        .deserialize_known_field("extra_solid_infills", "trailing\\")
        .unwrap_err();
    assert!(error.to_string().contains("extra_solid_infills"));
}

#[test]
fn string_vector_metadata_matches_orca_quoted_semicolon_state_machine() {
    let cases = [
        ("", vec![]),
        (r#""alpha;beta";gamma"#, vec!["alpha;beta", "gamma"]),
        (
            r#""quote\" slash\\ return\r newline\n generic\q""#,
            vec!["quote\" slash\\ return\r newline\n genericq"],
        ),
        (" \tfirst; \tsecond", vec!["first", "second"]),
        (" \t\"first\" \t; \tsecond", vec!["first", "second"]),
        ("alpha \t;beta", vec!["alpha \t", "beta"]),
        ("alpha;;", vec!["alpha", "", ""]),
        (";", vec!["", ""]),
    ];

    for (metadata, expected) in cases {
        let mut overrides = RegionOptionOverrides::default();
        assert!(overrides
            .deserialize_known_field("print_extruder_variant", metadata)
            .unwrap());
        assert_eq!(overrides.print_extruder_variant.unwrap().0, expected);
    }
}

#[test]
fn string_vector_metadata_rejects_malformed_quotes_and_escapes_with_key() {
    for malformed in ["\"unterminated", "\"closed\"garbage", "\"trailing\\"] {
        let error = RegionOptionOverrides::default()
            .deserialize_known_field("print_extruder_variant", malformed)
            .unwrap_err();
        assert!(
            error.to_string().contains("print_extruder_variant"),
            "{error}"
        );
    }
}
