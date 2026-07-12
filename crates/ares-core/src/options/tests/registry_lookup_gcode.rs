use crate::{OptionValueKind, option_definition};

#[test]
fn gcode_flavor_and_object_label_lookup_returns_upstream_metadata() {
    for (key, kind, default_value, source_fragments) in [
        (
            "exclude_object",
            OptionValueKind::Bool,
            "false",
            &["PrintConfig.hpp:1624", "PrintConfig.cpp:3839-3843"][..],
        ),
        (
            "gcode_comments",
            OptionValueKind::Bool,
            "false",
            &["PrintConfig.hpp:1626", "PrintConfig.cpp:3845-3851"][..],
        ),
        (
            "gcode_flavor",
            OptionValueKind::Enum,
            "marlin",
            &[
                "PrintConfig.hpp:33-46",
                "PrintConfig.hpp:1355",
                "PrintConfig.cpp:161-176",
                "PrintConfig.cpp:3785-3817",
            ][..],
        ),
        (
            "gcode_label_objects",
            OptionValueKind::Bool,
            "true",
            &["PrintConfig.hpp:1623", "PrintConfig.cpp:3831-3837"][..],
        ),
        (
            "pellet_modded_printer",
            OptionValueKind::Bool,
            "false",
            &["PrintConfig.cpp:3819-3823"][..],
        ),
        (
            "support_multi_bed_types",
            OptionValueKind::Bool,
            "false",
            &["PrintConfig.hpp:1461", "PrintConfig.cpp:3825-3829"][..],
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
