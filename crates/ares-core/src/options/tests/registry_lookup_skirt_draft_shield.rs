#[test]
fn exposes_skirt_draft_shield_option_definition_lookup() {
    for (key, kind, default_value, source_fragments) in [
        (
            "draft_shield",
            crate::OptionValueKind::Enum,
            "disabled",
            &[
                "PrintConfig.hpp:290-292",
                "PrintConfig.hpp:1512",
                "PrintConfig.cpp:443-447",
                "PrintConfig.cpp:5573-5586",
            ][..],
        ),
        (
            "min_skirt_length",
            crate::OptionValueKind::Float,
            "0",
            &["PrintConfig.hpp:1558", "PrintConfig.cpp:5618-5627"][..],
        ),
        (
            "single_loop_draft_shield",
            crate::OptionValueKind::Bool,
            "false",
            &["PrintConfig.hpp:1557", "PrintConfig.cpp:5567-5571"][..],
        ),
        (
            "skirt_distance",
            crate::OptionValueKind::Float,
            "2",
            &["PrintConfig.hpp:1552", "PrintConfig.cpp:5540-5547"][..],
        ),
        (
            "skirt_height",
            crate::OptionValueKind::Int,
            "1",
            &["PrintConfig.hpp:1553", "PrintConfig.cpp:5559-5565"][..],
        ),
        (
            "skirt_loops",
            crate::OptionValueKind::Int,
            "1",
            &["PrintConfig.hpp:1554", "PrintConfig.cpp:5600-5607"][..],
        ),
        (
            "skirt_speed",
            crate::OptionValueKind::Float,
            "50",
            &["PrintConfig.hpp:1556", "PrintConfig.cpp:5609-5616"][..],
        ),
        (
            "skirt_start_angle",
            crate::OptionValueKind::Float,
            "-135",
            &["PrintConfig.hpp:927", "PrintConfig.cpp:5549-5557"][..],
        ),
        (
            "skirt_type",
            crate::OptionValueKind::Enum,
            "combined",
            &[
                "PrintConfig.hpp:286-288",
                "PrintConfig.hpp:1555",
                "PrintConfig.cpp:437-441",
                "PrintConfig.cpp:5588-5598",
            ][..],
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
