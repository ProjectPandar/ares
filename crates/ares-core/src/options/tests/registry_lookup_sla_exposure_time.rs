#[test]
fn exposes_sla_exposure_time_option_definition_lookup() {
    for (key, kind, default_value, source_fragments) in [
        (
            "exposure_time",
            crate::OptionValueKind::Float,
            "10",
            &["PrintConfig.hpp:1815", "PrintConfig.cpp:7449-7454"][..],
        ),
        (
            "faded_layers",
            crate::OptionValueKind::Int,
            "10",
            &["PrintConfig.cpp:7425-7431"][..],
        ),
        (
            "initial_exposure_time",
            crate::OptionValueKind::Float,
            "15",
            &["PrintConfig.hpp:1816", "PrintConfig.cpp:7472-7477"][..],
        ),
        (
            "max_exposure_time",
            crate::OptionValueKind::Float,
            "100",
            &["PrintConfig.hpp:1849", "PrintConfig.cpp:7441-7447"][..],
        ),
        (
            "max_initial_exposure_time",
            crate::OptionValueKind::Float,
            "150",
            &["PrintConfig.hpp:1851", "PrintConfig.cpp:7464-7470"][..],
        ),
        (
            "min_exposure_time",
            crate::OptionValueKind::Float,
            "0",
            &["PrintConfig.hpp:1848", "PrintConfig.cpp:7433-7439"][..],
        ),
        (
            "min_initial_exposure_time",
            crate::OptionValueKind::Float,
            "0",
            &["PrintConfig.hpp:1850", "PrintConfig.cpp:7456-7462"][..],
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
