#[test]
fn exposes_head_wrap_thin_wall_option_definition_lookup() {
    for (key, kind, default_value, source_fragments) in [
        (
            "head_wrap_detect_zone",
            crate::OptionValueKind::Points,
            "0x0",
            &["PrintConfig.hpp:1485", "PrintConfig.cpp:6503-6506"][..],
        ),
        (
            "detect_thin_wall",
            crate::OptionValueKind::Bool,
            "false",
            &["PrintConfig.hpp:1165", "PrintConfig.cpp:6508-6514"][..],
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
fn detect_thin_wall_runtime_option_defaults_to_false_and_accepts_true() {
    let defaults = crate::SliceOptions::default();
    let enabled: crate::SliceOptions =
        serde_json::from_value(serde_json::json!({ "detect_thin_wall": true })).unwrap();

    assert!(!defaults.perimeter_options().unwrap().detect_thin_wall());
    assert!(enabled.perimeter_options().unwrap().detect_thin_wall());
}

#[test]
fn detect_thin_wall_rejects_non_bool_runtime_value() {
    let options: crate::SliceOptions =
        serde_json::from_value(serde_json::json!({ "detect_thin_wall": "true" })).unwrap();

    assert!(matches!(
        options.perimeter_options(),
        Err(crate::SliceError::InvalidInput(message)) if message.contains("detect_thin_wall")
    ));
}
