#[test]
fn exposes_change_gcode_option_definition_lookup() {
    for (key, kind, default_value, source_fragments) in [
        (
            "change_extrusion_role_gcode",
            crate::OptionValueKind::String,
            "",
            &["PrintConfig.hpp:1393", "PrintConfig.cpp:6525-6532"][..],
        ),
        (
            "change_filament_gcode",
            crate::OptionValueKind::String,
            "",
            &["PrintConfig.hpp:1392", "PrintConfig.cpp:6516-6523"][..],
        ),
        (
            "filament_change_extrusion_role_gcode",
            crate::OptionValueKind::Strings,
            "",
            &["PrintConfig.hpp:1395", "PrintConfig.cpp:6534-6541"][..],
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
