#[test]
fn exposes_overhang_wall_option_definition_lookup() {
    for (key, kind, default_value, source_fragments) in [
        (
            "detect_overhang_wall",
            crate::OptionValueKind::Bool,
            "true",
            &["PrintConfig.hpp:1153", "PrintConfig.cpp:4879-4885"][..],
        ),
        (
            "inner_wall_line_width",
            crate::OptionValueKind::FloatOrPercent,
            "0",
            &["PrintConfig.hpp:1155", "PrintConfig.cpp:4896-4906"][..],
        ),
        (
            "inner_wall_speed",
            crate::OptionValueKind::Float,
            "60",
            &["PrintConfig.hpp:1156", "PrintConfig.cpp:4908-4916"][..],
        ),
        (
            "make_overhang_printable",
            crate::OptionValueKind::Bool,
            "false",
            &["PrintConfig.hpp:1199", "PrintConfig.cpp:4850-4855"][..],
        ),
        (
            "make_overhang_printable_angle",
            crate::OptionValueKind::Float,
            "55",
            &["PrintConfig.hpp:1032", "PrintConfig.cpp:4857-4867"][..],
        ),
        (
            "make_overhang_printable_hole_size",
            crate::OptionValueKind::Float,
            "0",
            &["PrintConfig.hpp:1033", "PrintConfig.cpp:4869-4877"][..],
        ),
        (
            "wall_filament",
            crate::OptionValueKind::Int,
            "1",
            &["PrintConfig.hpp:1154", "PrintConfig.cpp:4887-4894"][..],
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
