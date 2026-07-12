#[test]
fn exposes_resonance_input_shaping_option_definition_lookup() {
    for (key, kind, default_value, source_fragments) in [
        (
            "input_shaping_damp_x",
            crate::OptionValueKind::Float,
            "0.1",
            &["PrintConfig.hpp:1286", "PrintConfig.cpp:4575-4581"][..],
        ),
        (
            "input_shaping_damp_y",
            crate::OptionValueKind::Float,
            "0.1",
            &["PrintConfig.hpp:1287", "PrintConfig.cpp:4583-4589"][..],
        ),
        (
            "input_shaping_emit",
            crate::OptionValueKind::Bool,
            "false",
            &["PrintConfig.hpp:1282", "PrintConfig.cpp:4541-4546"][..],
        ),
        (
            "input_shaping_freq_x",
            crate::OptionValueKind::Float,
            "0",
            &["PrintConfig.hpp:1284", "PrintConfig.cpp:4557-4564"][..],
        ),
        (
            "input_shaping_freq_y",
            crate::OptionValueKind::Float,
            "0",
            &["PrintConfig.hpp:1285", "PrintConfig.cpp:4566-4573"][..],
        ),
        (
            "input_shaping_type",
            crate::OptionValueKind::Enum,
            "Default",
            &[
                "PrintConfig.hpp:365-379",
                "PrintConfig.hpp:544",
                "PrintConfig.hpp:1283",
                "PrintConfig.cpp:503-518",
                "PrintConfig.cpp:4548-4555",
            ][..],
        ),
        (
            "max_resonance_avoidance_speed",
            crate::OptionValueKind::Float,
            "120",
            &["PrintConfig.hpp:1279", "PrintConfig.cpp:4533-4539"][..],
        ),
        (
            "min_resonance_avoidance_speed",
            crate::OptionValueKind::Float,
            "70",
            &["PrintConfig.hpp:1278", "PrintConfig.cpp:4525-4531"][..],
        ),
        (
            "resonance_avoidance",
            crate::OptionValueKind::Bool,
            "false",
            &["PrintConfig.hpp:1277", "PrintConfig.cpp:4516-4523"][..],
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
