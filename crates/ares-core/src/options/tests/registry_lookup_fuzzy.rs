#[test]
fn exposes_fuzzy_skin_option_definition_lookup() {
    for (key, kind, default_value, source_fragments) in [
        (
            "fuzzy_skin",
            crate::OptionValueKind::Enum,
            "disabled_fuzzy",
            &[
                "PrintConfig.hpp:50-57",
                "PrintConfig.hpp:1108",
                "PrintConfig.cpp:192-200",
                "PrintConfig.cpp:3420-3439",
            ][..],
        ),
        (
            "fuzzy_skin_first_layer",
            crate::OptionValueKind::Bool,
            "false",
            &["PrintConfig.hpp:1111", "PrintConfig.cpp:3461-3466"][..],
        ),
        (
            "fuzzy_skin_layers_between_ripple_offset",
            crate::OptionValueKind::Int,
            "1",
            &["PrintConfig.hpp:1119", "PrintConfig.cpp:3567-3576"][..],
        ),
        (
            "fuzzy_skin_mode",
            crate::OptionValueKind::Enum,
            "displacement",
            &[
                "PrintConfig.hpp:59-63",
                "PrintConfig.hpp:1113",
                "PrintConfig.cpp:218-223",
                "PrintConfig.cpp:3468-3489",
            ][..],
        ),
        (
            "fuzzy_skin_noise_type",
            crate::OptionValueKind::Enum,
            "classic",
            &[
                "PrintConfig.hpp:65-72",
                "PrintConfig.hpp:1112",
                "PrintConfig.cpp:202-210",
                "PrintConfig.cpp:3491-3515",
            ][..],
        ),
        (
            "fuzzy_skin_octaves",
            crate::OptionValueKind::Int,
            "4",
            &["PrintConfig.hpp:1115", "PrintConfig.cpp:3527-3534"][..],
        ),
        (
            "fuzzy_skin_persistence",
            crate::OptionValueKind::Float,
            "0.5",
            &["PrintConfig.hpp:1116", "PrintConfig.cpp:3536-3543"][..],
        ),
        (
            "fuzzy_skin_point_distance",
            crate::OptionValueKind::Float,
            "0.3",
            &["PrintConfig.hpp:1110", "PrintConfig.cpp:3451-3459"][..],
        ),
        (
            "fuzzy_skin_ripple_offset",
            crate::OptionValueKind::Percent,
            "50",
            &["PrintConfig.hpp:1118", "PrintConfig.cpp:3553-3565"][..],
        ),
        (
            "fuzzy_skin_ripples_per_layer",
            crate::OptionValueKind::Int,
            "15",
            &["PrintConfig.hpp:1117", "PrintConfig.cpp:3545-3551"][..],
        ),
        (
            "fuzzy_skin_scale",
            crate::OptionValueKind::Float,
            "1",
            &["PrintConfig.hpp:1114", "PrintConfig.cpp:3517-3525"][..],
        ),
        (
            "fuzzy_skin_thickness",
            crate::OptionValueKind::Float,
            "0.2",
            &["PrintConfig.hpp:1109", "PrintConfig.cpp:3441-3449"][..],
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
