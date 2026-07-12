#[test]
fn exposes_priming_slicing_support_option_definition_lookup() {
    for (key, kind, default_value, source_fragments) in [
        (
            "enable_support",
            crate::OptionValueKind::Bool,
            "false",
            &["PrintConfig.hpp:948", "PrintConfig.cpp:5903-5908"][..],
        ),
        (
            "single_extruder_multi_material_priming",
            crate::OptionValueKind::Bool,
            "false",
            &["PrintConfig.hpp:1390", "PrintConfig.cpp:5863-5867"][..],
        ),
        (
            "slice_closing_radius",
            crate::OptionValueKind::Float,
            "0.049",
            &["PrintConfig.hpp:946", "PrintConfig.cpp:5869-5877"][..],
        ),
        (
            "slicing_mode",
            crate::OptionValueKind::Enum,
            "regular",
            &[
                "PrintConfig.hpp:162-170",
                "PrintConfig.hpp:947",
                "PrintConfig.cpp:305-310",
                "PrintConfig.cpp:5879-5891",
            ][..],
        ),
        (
            "z_offset",
            crate::OptionValueKind::Float,
            "0",
            &["PrintConfig.hpp:1609", "PrintConfig.cpp:5893-5901"][..],
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
