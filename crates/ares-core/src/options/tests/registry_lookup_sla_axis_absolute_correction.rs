#[test]
fn exposes_sla_axis_absolute_correction_option_definition_lookup() {
    for (key, kind, default_value, source_fragments) in [
        (
            "absolute_correction",
            crate::OptionValueKind::Float,
            "0",
            &["PrintConfig.hpp:1841", "PrintConfig.cpp:7344-7349"][..],
        ),
        (
            "relative_correction_x",
            crate::OptionValueKind::Float,
            "1",
            &["PrintConfig.hpp:1838", "PrintConfig.cpp:7320-7326"][..],
        ),
        (
            "relative_correction_y",
            crate::OptionValueKind::Float,
            "1",
            &["PrintConfig.hpp:1839", "PrintConfig.cpp:7328-7334"][..],
        ),
        (
            "relative_correction_z",
            crate::OptionValueKind::Float,
            "1",
            &["PrintConfig.hpp:1840", "PrintConfig.cpp:7336-7342"][..],
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
