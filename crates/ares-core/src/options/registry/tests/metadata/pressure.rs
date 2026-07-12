use super::super::super::{OptionValueKind, option_definition};

#[test]
fn pressure_advance_metadata_matches_upstream_print_config() {
    for (key, kind, default_value, source_fragments) in [
        (
            "adaptive_pressure_advance",
            OptionValueKind::Bools,
            "false",
            &["PrintConfig.hpp:1305", "PrintConfig.cpp:2264-2278"][..],
        ),
        (
            "adaptive_pressure_advance_bridges",
            OptionValueKind::Floats,
            "0",
            &["PrintConfig.hpp:1308", "PrintConfig.cpp:2313-2320"][..],
        ),
        (
            "adaptive_pressure_advance_model",
            OptionValueKind::Strings,
            "0,0,0\n0,0,0",
            &["PrintConfig.hpp:1307", "PrintConfig.cpp:2280-2303"][..],
        ),
        (
            "adaptive_pressure_advance_overhangs",
            OptionValueKind::Bools,
            "false",
            &["PrintConfig.hpp:1306", "PrintConfig.cpp:2305-2311"][..],
        ),
        (
            "enable_pressure_advance",
            OptionValueKind::Bools,
            "false",
            &["PrintConfig.hpp:1302", "PrintConfig.cpp:2252-2255"][..],
        ),
        (
            "pressure_advance",
            OptionValueKind::Floats,
            "0.02",
            &["PrintConfig.hpp:1303", "PrintConfig.cpp:2257-2262"][..],
        ),
    ] {
        let definition = option_definition(key).unwrap();
        assert_eq!(definition.kind, kind);
        assert_eq!(definition.default_value, default_value);
        for fragment in source_fragments {
            assert!(definition.source.contains(fragment));
        }
    }
}
