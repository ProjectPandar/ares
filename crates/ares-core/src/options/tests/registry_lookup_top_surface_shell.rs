#[test]
fn exposes_top_surface_shell_option_definition_lookup() {
    for (key, kind, default_value, source_fragments) in [
        (
            "top_shell_layers",
            crate::OptionValueKind::Int,
            "4",
            &["PrintConfig.hpp:1167", "PrintConfig.cpp:6564-6573"][..],
        ),
        (
            "top_shell_thickness",
            crate::OptionValueKind::Float,
            "0.6",
            &["PrintConfig.hpp:1168", "PrintConfig.cpp:6575-6584"][..],
        ),
        (
            "top_surface_line_width",
            crate::OptionValueKind::FloatOrPercent,
            "0",
            &["PrintConfig.hpp:1166", "PrintConfig.cpp:6543-6553"][..],
        ),
        (
            "top_surface_speed",
            crate::OptionValueKind::Float,
            "100",
            &["PrintConfig.hpp:1169", "PrintConfig.cpp:6555-6562"][..],
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
