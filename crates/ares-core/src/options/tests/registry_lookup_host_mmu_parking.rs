#[test]
fn exposes_host_mmu_parking_option_definition_lookup() {
    for (key, kind, default_value, source_fragments) in [
        (
            "cooling_tube_length",
            crate::OptionValueKind::Float,
            "5",
            &["PrintConfig.hpp:1429", "PrintConfig.cpp:4787-4793"][..],
        ),
        (
            "cooling_tube_retraction",
            crate::OptionValueKind::Float,
            "91.5",
            &["PrintConfig.hpp:1428", "PrintConfig.cpp:4779-4785"][..],
        ),
        (
            "high_current_on_filament_swap",
            crate::OptionValueKind::Bool,
            "false",
            &["PrintConfig.hpp:1430", "PrintConfig.cpp:4795-4801"][..],
        ),
        (
            "host_type",
            crate::OptionValueKind::Enum,
            "octoprint",
            &[
                "PrintConfig.hpp:79-81",
                "PrintConfig.cpp:137-153",
                "PrintConfig.cpp:4733-4768",
            ][..],
        ),
        (
            "notes",
            crate::OptionValueKind::String,
            "",
            &["PrintConfig.hpp:1633", "PrintConfig.cpp:4723-4731"][..],
        ),
        (
            "nozzle_volume",
            crate::OptionValueKind::FloatsNullable,
            "0.0",
            &["PrintConfig.hpp:1613", "PrintConfig.cpp:4770-4777"][..],
        ),
        (
            "parking_pos_retraction",
            crate::OptionValueKind::Float,
            "92",
            &["PrintConfig.hpp:1431", "PrintConfig.cpp:4803-4810"][..],
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
