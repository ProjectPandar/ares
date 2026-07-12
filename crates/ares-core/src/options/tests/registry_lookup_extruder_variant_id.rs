#[test]
fn exposes_extruder_variant_id_option_definition_lookup() {
    for (key, kind, default_value, source_fragments) in [
        (
            "extruder_ams_count",
            crate::OptionValueKind::Strings,
            "",
            &["PrintConfig.hpp:1410", "PrintConfig.cpp:5246-5250"][..],
        ),
        (
            "extruder_variant_list",
            crate::OptionValueKind::Strings,
            "Direct Drive Standard",
            &["PrintConfig.cpp:5239-5244"][..],
        ),
        (
            "filament_extruder_variant",
            crate::OptionValueKind::Strings,
            "Direct Drive Standard",
            &["PrintConfig.hpp:1338", "PrintConfig.cpp:5292-5297"][..],
        ),
        (
            "filament_self_index",
            crate::OptionValueKind::Ints,
            "1",
            &["PrintConfig.cpp:5299-5304"][..],
        ),
        (
            "master_extruder_id",
            crate::OptionValueKind::Int,
            "1",
            &["PrintConfig.hpp:1412", "PrintConfig.cpp:5266-5270"][..],
        ),
        (
            "print_extruder_id",
            crate::OptionValueKind::Ints,
            "1",
            &["PrintConfig.hpp:1077", "PrintConfig.cpp:5272-5277"][..],
        ),
        (
            "print_extruder_variant",
            crate::OptionValueKind::Strings,
            "Direct Drive Standard",
            &["PrintConfig.hpp:1078", "PrintConfig.cpp:5279-5284"][..],
        ),
        (
            "printer_extruder_id",
            crate::OptionValueKind::Ints,
            "1",
            &["PrintConfig.hpp:1411", "PrintConfig.cpp:5252-5257"][..],
        ),
        (
            "printer_extruder_variant",
            crate::OptionValueKind::Strings,
            "Direct Drive Standard",
            &["PrintConfig.hpp:1413", "PrintConfig.cpp:5259-5264"][..],
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
