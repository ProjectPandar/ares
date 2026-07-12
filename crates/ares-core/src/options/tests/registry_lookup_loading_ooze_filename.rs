#[test]
fn exposes_loading_ooze_filename_option_definition_lookup() {
    for (key, kind, default_value, source_fragments) in [
        (
            "extra_loading_move",
            crate::OptionValueKind::Float,
            "-2",
            &["PrintConfig.hpp:1432", "PrintConfig.cpp:4812-4819"][..],
        ),
        (
            "filename_format",
            crate::OptionValueKind::String,
            "{input_filename_base}_{filament_type[initial_tool]}_{print_time}.gcode",
            &["PrintConfig.hpp:1546", "PrintConfig.cpp:4843-4848"][..],
        ),
        (
            "ooze_prevention",
            crate::OptionValueKind::Bool,
            "false",
            &["PrintConfig.hpp:1545", "PrintConfig.cpp:4837-4841"][..],
        ),
        (
            "reduce_infill_retraction",
            crate::OptionValueKind::Bool,
            "false",
            &["PrintConfig.hpp:1544", "PrintConfig.cpp:4829-4835"][..],
        ),
        (
            "start_end_points",
            crate::OptionValueKind::Points,
            "30x-3,54x245",
            &["PrintConfig.hpp:1614", "PrintConfig.cpp:4821-4827"][..],
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
