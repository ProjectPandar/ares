#[test]
fn exposes_wall_post_process_printer_option_definition_lookup() {
    for (key, kind, default_value, source_fragments) in [
        (
            "alternate_extra_wall",
            crate::OptionValueKind::Bool,
            "false",
            &["PrintConfig.hpp:1159", "PrintConfig.cpp:4926-4933"][..],
        ),
        (
            "post_process",
            crate::OptionValueKind::Strings,
            "",
            &["PrintConfig.hpp:1547", "PrintConfig.cpp:4935-4946"][..],
        ),
        (
            "print_settings_id",
            crate::OptionValueKind::String,
            "",
            &["PrintConfig.cpp:4978-4981"][..],
        ),
        (
            "printer_model",
            crate::OptionValueKind::String,
            "",
            &["PrintConfig.hpp:1548", "PrintConfig.cpp:4957-4961"][..],
        ),
        (
            "printer_notes",
            crate::OptionValueKind::String,
            "",
            &["PrintConfig.hpp:1634", "PrintConfig.cpp:4963-4970"][..],
        ),
        (
            "printer_settings_id",
            crate::OptionValueKind::String,
            "",
            &["PrintConfig.cpp:4983-4986"][..],
        ),
        (
            "printer_variant",
            crate::OptionValueKind::String,
            "",
            &["PrintConfig.cpp:4972-4976"][..],
        ),
        (
            "process_change_extrusion_role_gcode",
            crate::OptionValueKind::String,
            "",
            &["PrintConfig.hpp:1394", "PrintConfig.cpp:4948-4955"][..],
        ),
        (
            "wall_loops",
            crate::OptionValueKind::Int,
            "2",
            &["PrintConfig.hpp:1158", "PrintConfig.cpp:4918-4924"][..],
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
