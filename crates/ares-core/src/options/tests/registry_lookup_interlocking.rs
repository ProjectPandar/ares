use crate::{OptionValueKind, option_definition};

#[test]
fn inheritance_interlocking_lookup_returns_upstream_metadata() {
    for (key, kind, default_value, source_fragments) in [
        (
            "calib_flowrate_topinfill_special_order",
            OptionValueKind::Bool,
            "false",
            &["PrintConfig.hpp:1070", "PrintConfig.cpp:4156-4159"][..],
        ),
        (
            "inherits",
            OptionValueKind::String,
            "",
            &["PrintConfig.cpp:4063-4069"][..],
        ),
        (
            "inherits_group",
            OptionValueKind::Strings,
            "",
            &["PrintConfig.cpp:4071-4075"][..],
        ),
        (
            "interface_shells",
            OptionValueKind::Bool,
            "false",
            &["PrintConfig.hpp:935", "PrintConfig.cpp:4077-4084"][..],
        ),
        (
            "interlocking_beam",
            OptionValueKind::Bool,
            "false",
            &["PrintConfig.hpp:1062", "PrintConfig.cpp:4106-4111"][..],
        ),
        (
            "interlocking_beam_layer_count",
            OptionValueKind::Int,
            "2",
            &["PrintConfig.hpp:1065", "PrintConfig.cpp:4132-4138"][..],
        ),
        (
            "interlocking_beam_width",
            OptionValueKind::Float,
            "0.8",
            &["PrintConfig.hpp:1063", "PrintConfig.cpp:4113-4120"][..],
        ),
        (
            "interlocking_boundary_avoidance",
            OptionValueKind::Int,
            "2",
            &["PrintConfig.hpp:1067", "PrintConfig.cpp:4148-4154"][..],
        ),
        (
            "interlocking_depth",
            OptionValueKind::Int,
            "2",
            &["PrintConfig.hpp:1066", "PrintConfig.cpp:4140-4146"][..],
        ),
        (
            "interlocking_orientation",
            OptionValueKind::Float,
            "22.5",
            &["PrintConfig.hpp:1064", "PrintConfig.cpp:4122-4130"][..],
        ),
        (
            "mmu_segmented_region_interlocking_depth",
            OptionValueKind::Float,
            "0",
            &["PrintConfig.hpp:938", "PrintConfig.cpp:4095-4104"][..],
        ),
        (
            "mmu_segmented_region_max_width",
            OptionValueKind::Float,
            "0",
            &["PrintConfig.hpp:937", "PrintConfig.cpp:4086-4093"][..],
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
