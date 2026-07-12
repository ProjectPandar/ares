#[test]
fn exposes_min_feature_length_option_definition_lookup() {
    for (key, kind, default_value, source_fragments) in [
        (
            "min_feature_size",
            crate::OptionValueKind::Percent,
            "25",
            &["PrintConfig.hpp:1025", "PrintConfig.cpp:7051-7060"][..],
        ),
        (
            "min_length_factor",
            crate::OptionValueKind::Float,
            "0.5",
            &["PrintConfig.hpp:1039", "PrintConfig.cpp:7062-7074"][..],
        ),
    ] {
        let definition = crate::option_definition(key).unwrap();
        assert_eq!(definition.kind, kind);
        assert_eq!(definition.default_value, default_value);
        for fragment in source_fragments {
            assert!(definition.source.contains(fragment));
        }
    }
}

#[test]
fn min_length_factor_runtime_defaults_to_orca_default() {
    let options = crate::SliceOptions::default();

    assert_eq!(
        options.perimeter_options().unwrap().min_length_factor(),
        0.5
    );
}

#[test]
fn min_length_factor_accepts_numeric_and_string_numeric_values() {
    for (value, expected) in [
        (serde_json::json!(2.25), 2.25),
        (serde_json::json!("3.5"), 3.5),
    ] {
        let options: crate::SliceOptions =
            serde_json::from_value(serde_json::json!({ "min_length_factor": value })).unwrap();

        assert_eq!(
            options.perimeter_options().unwrap().min_length_factor(),
            expected
        );
    }
}

#[test]
fn min_length_factor_rejects_out_of_range_non_finite_and_non_numeric_values() {
    for value in [
        serde_json::json!(-0.1),
        serde_json::json!(25.1),
        serde_json::json!("NaN"),
        serde_json::json!("inf"),
        serde_json::json!("wide"),
        serde_json::json!(true),
    ] {
        let options: crate::SliceOptions =
            serde_json::from_value(serde_json::json!({ "min_length_factor": value })).unwrap();

        assert!(matches!(
            options.perimeter_options(),
            Err(crate::SliceError::InvalidInput(message)) if message.contains("min_length_factor")
        ));
    }
}
