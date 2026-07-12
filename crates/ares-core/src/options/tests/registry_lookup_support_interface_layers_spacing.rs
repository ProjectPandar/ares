#[test]
fn exposes_support_interface_layers_spacing_option_definition_lookup() {
    for (key, kind, default_value, source_fragments) in [
        (
            "support_interface_bottom_layers",
            crate::OptionValueKind::Int,
            "0",
            &["PrintConfig.hpp:965", "PrintConfig.cpp:6090-6102"][..],
        ),
        (
            "support_interface_filament",
            crate::OptionValueKind::Int,
            "0",
            &["PrintConfig.hpp:963", "PrintConfig.cpp:6062-6070"][..],
        ),
        (
            "support_interface_loop_pattern",
            crate::OptionValueKind::Bool,
            "false",
            &["PrintConfig.hpp:962", "PrintConfig.cpp:6055-6060"][..],
        ),
        (
            "support_interface_spacing",
            crate::OptionValueKind::Float,
            "0.5",
            &["PrintConfig.hpp:966-967", "PrintConfig.cpp:6104-6112"][..],
        ),
        (
            "support_interface_top_layers",
            crate::OptionValueKind::Int,
            "3",
            &["PrintConfig.hpp:964", "PrintConfig.cpp:6072-6088"][..],
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
