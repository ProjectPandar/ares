#[test]
fn exposes_start_gcode_filament_change_option_definition_lookup() {
    for (key, kind, default_value, source_fragments) in [
        (
            "file_start_gcode",
            crate::OptionValueKind::String,
            "",
            &["PrintConfig.hpp:1385", "PrintConfig.cpp:5777-5787"][..],
        ),
        (
            "filament_start_gcode",
            crate::OptionValueKind::Strings,
            " ",
            &["PrintConfig.hpp:1387", "PrintConfig.cpp:5798-5805"][..],
        ),
        (
            "machine_start_gcode",
            crate::OptionValueKind::String,
            "G28 ; home all axes\nG1 Z5 F5000 ; lift nozzle\n",
            &["PrintConfig.hpp:1386", "PrintConfig.cpp:5789-5796"][..],
        ),
        (
            "manual_filament_change",
            crate::OptionValueKind::Bool,
            "false",
            &["PrintConfig.hpp:1389", "PrintConfig.cpp:5813-5819"][..],
        ),
        (
            "single_extruder_multi_material",
            crate::OptionValueKind::Bool,
            "true",
            &["PrintConfig.hpp:1388", "PrintConfig.cpp:5807-5811"][..],
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
