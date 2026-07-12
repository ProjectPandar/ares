#[test]
fn exposes_sla_display_tilt_option_definition_lookup() {
    for (key, kind, default_value, source_fragments) in [
        (
            "area_fill",
            crate::OptionValueKind::Float,
            "50",
            &["PrintConfig.hpp:1847", "PrintConfig.cpp:7304-7310"][..],
        ),
        (
            "display_height",
            crate::OptionValueKind::Float,
            "68",
            &["PrintConfig.hpp:1831", "PrintConfig.cpp:7241-7245"][..],
        ),
        (
            "display_mirror_x",
            crate::OptionValueKind::Bool,
            "true",
            &["PrintConfig.hpp:1835", "PrintConfig.cpp:7261-7266"][..],
        ),
        (
            "display_mirror_y",
            crate::OptionValueKind::Bool,
            "false",
            &["PrintConfig.hpp:1836", "PrintConfig.cpp:7268-7273"][..],
        ),
        (
            "display_orientation",
            crate::OptionValueKind::Enum,
            "portrait",
            &[
                "PrintConfig.hpp:260-263",
                "PrintConfig.hpp:1834",
                "PrintConfig.cpp:400-404",
                "PrintConfig.cpp:7275-7284",
            ][..],
        ),
        (
            "display_pixels_x",
            crate::OptionValueKind::Int,
            "2560",
            &["PrintConfig.hpp:1832", "PrintConfig.cpp:7247-7252"][..],
        ),
        (
            "display_pixels_y",
            crate::OptionValueKind::Int,
            "1440",
            &["PrintConfig.hpp:1833", "PrintConfig.cpp:7254-7259"][..],
        ),
        (
            "display_width",
            crate::OptionValueKind::Float,
            "120",
            &["PrintConfig.hpp:1830", "PrintConfig.cpp:7235-7239"][..],
        ),
        (
            "fast_tilt_time",
            crate::OptionValueKind::Float,
            "5",
            &["PrintConfig.hpp:1845", "PrintConfig.cpp:7286-7293"][..],
        ),
        (
            "slow_tilt_time",
            crate::OptionValueKind::Float,
            "8",
            &["PrintConfig.hpp:1846", "PrintConfig.cpp:7295-7302"][..],
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
