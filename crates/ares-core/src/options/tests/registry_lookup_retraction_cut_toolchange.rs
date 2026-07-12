#[test]
fn exposes_retraction_cut_toolchange_option_definition_lookup() {
    for (key, kind, default_value, source_fragments) in [
        (
            "enable_long_retraction_when_cut",
            crate::OptionValueKind::Int,
            "0",
            &["PrintConfig.hpp:1370", "PrintConfig.cpp:5077-5079"][..],
        ),
        (
            "long_retractions_when_cut",
            crate::OptionValueKind::Bools,
            "false",
            &["PrintConfig.hpp:1372", "PrintConfig.cpp:5081-5086"][..],
        ),
        (
            "long_retractions_when_ec",
            crate::OptionValueKind::BoolsNullable,
            "false",
            &["PrintConfig.hpp:1374", "PrintConfig.cpp:5096-5100"][..],
        ),
        (
            "retract_length_toolchange",
            crate::OptionValueKind::Floats,
            "10",
            &["PrintConfig.hpp:1369", "PrintConfig.cpp:5111-5120"][..],
        ),
        (
            "retraction_distances_when_cut",
            crate::OptionValueKind::Floats,
            "18",
            &["PrintConfig.hpp:1371", "PrintConfig.cpp:5088-5094"][..],
        ),
        (
            "retraction_distances_when_ec",
            crate::OptionValueKind::FloatsNullable,
            "10",
            &["PrintConfig.hpp:1373", "PrintConfig.cpp:5102-5109"][..],
        ),
        (
            "retraction_length",
            crate::OptionValueKind::Floats,
            "0.8",
            &["PrintConfig.hpp:1368", "PrintConfig.cpp:5068-5075"][..],
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
