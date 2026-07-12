#[test]
fn exposes_process_gcode_utility_option_definition_lookup() {
    for (key, kind, default_value, source_fragments) in [
        (
            "enable_arc_fitting",
            crate::OptionValueKind::Bool,
            "false",
            &["PrintConfig.hpp:1298", "PrintConfig.cpp:3607-3616"][..],
        ),
        (
            "enable_power_loss_recovery",
            crate::OptionValueKind::Enum,
            "printer_configuration",
            &[
                "PrintConfig.hpp:125-129",
                "PrintConfig.hpp:1347",
                "PrintConfig.cpp:185-190",
                "PrintConfig.cpp:3632-3643",
            ][..],
        ),
        (
            "filter_out_gap_fill",
            crate::OptionValueKind::Float,
            "0",
            &["PrintConfig.hpp:1190", "PrintConfig.cpp:3578-3585"][..],
        ),
        (
            "gap_infill_speed",
            crate::OptionValueKind::Float,
            "30",
            &["PrintConfig.hpp:1120", "PrintConfig.cpp:3587-3594"][..],
        ),
        (
            "gcode_add_line_number",
            crate::OptionValueKind::Bool,
            "false",
            &["PrintConfig.hpp:1353", "PrintConfig.cpp:3618-3622"][..],
        ),
        (
            "precise_z_height",
            crate::OptionValueKind::Bool,
            "false",
            &["PrintConfig.hpp:1059", "PrintConfig.cpp:3597-3604"][..],
        ),
        (
            "scan_first_layer",
            crate::OptionValueKind::Bool,
            "false",
            &["PrintConfig.hpp:1346", "PrintConfig.cpp:3625-3629"][..],
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
