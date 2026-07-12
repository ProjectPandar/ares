#[test]
fn exposes_wipe_speed_loop_option_definition_lookup() {
    for (key, kind, default_value, source_fragments) in [
        (
            "role_based_wipe_speed",
            crate::OptionValueKind::Bool,
            "true",
            &["PrintConfig.hpp:1183", "PrintConfig.cpp:5502-5508"][..],
        ),
        (
            "wipe_before_external_loop",
            crate::OptionValueKind::Bool,
            "false",
            &["PrintConfig.hpp:1186", "PrintConfig.cpp:5517-5526"][..],
        ),
        (
            "wipe_on_loops",
            crate::OptionValueKind::Bool,
            "false",
            &["PrintConfig.hpp:1185", "PrintConfig.cpp:5510-5515"][..],
        ),
        (
            "wipe_speed",
            crate::OptionValueKind::FloatOrPercent,
            "80%",
            &["PrintConfig.hpp:1184", "PrintConfig.cpp:5528-5538"][..],
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
