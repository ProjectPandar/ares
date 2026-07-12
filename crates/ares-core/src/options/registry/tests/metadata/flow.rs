use super::super::super::{OptionValueKind, option_definition};

#[test]
fn flow_ratio_metadata_matches_upstream_print_config() {
    for (key, kind, default_value, source_fragments) in [
        (
            "filament_flow_ratio",
            OptionValueKind::FloatsNullable,
            "1",
            &["PrintConfig.hpp:1301", "PrintConfig.cpp:2227-2237"][..],
        ),
        (
            "print_flow_ratio",
            OptionValueKind::Float,
            "1",
            &["PrintConfig.cpp:2239-2250"][..],
        ),
        (
            "top_solid_infill_flow_ratio",
            OptionValueKind::Float,
            "1",
            &["PrintConfig.hpp:1193", "PrintConfig.cpp:1286"][..],
        ),
        (
            "first_layer_flow_ratio",
            OptionValueKind::Float,
            "1",
            &["PrintConfig.hpp:1215", "PrintConfig.cpp:1314"][..],
        ),
        (
            "inner_wall_flow_ratio",
            OptionValueKind::Float,
            "1",
            &["PrintConfig.hpp:1217", "PrintConfig.cpp:1334"][..],
        ),
        (
            "outer_wall_flow_ratio",
            OptionValueKind::Float,
            "1",
            &["PrintConfig.hpp:1216", "PrintConfig.cpp:1324"][..],
        ),
        (
            "overhang_flow_ratio",
            OptionValueKind::Float,
            "1",
            &["PrintConfig.hpp:1218", "PrintConfig.cpp:1344"][..],
        ),
        (
            "set_other_flow_ratios",
            OptionValueKind::Bool,
            "false",
            &["PrintConfig.hpp:978", "PrintConfig.cpp:1307"][..],
        ),
        (
            "sparse_infill_flow_ratio",
            OptionValueKind::Float,
            "1",
            &["PrintConfig.hpp:1219", "PrintConfig.cpp:1354"][..],
        ),
        (
            "support_flow_ratio",
            OptionValueKind::Float,
            "1",
            &["PrintConfig.hpp:980", "PrintConfig.cpp:1384"][..],
        ),
        (
            "support_interface_flow_ratio",
            OptionValueKind::Float,
            "1",
            &["PrintConfig.hpp:981", "PrintConfig.cpp:1394"][..],
        ),
        (
            "internal_solid_infill_flow_ratio",
            OptionValueKind::Float,
            "1",
            &["PrintConfig.hpp:1220", "PrintConfig.cpp:1364"][..],
        ),
        (
            "gap_fill_flow_ratio",
            OptionValueKind::Float,
            "1",
            &["PrintConfig.hpp:1221", "PrintConfig.cpp:1374"][..],
        ),
        (
            "bottom_solid_infill_flow_ratio",
            OptionValueKind::Float,
            "1",
            &["PrintConfig.hpp:1194", "PrintConfig.cpp:1297"][..],
        ),
        (
            "bridge_angle",
            OptionValueKind::Float,
            "0",
            &["PrintConfig.hpp:1081", "PrintConfig.cpp:1213"][..],
        ),
        (
            "internal_bridge_angle",
            OptionValueKind::Float,
            "0",
            &["PrintConfig.hpp:1082", "PrintConfig.cpp:1226"][..],
        ),
        (
            "bridge_density",
            OptionValueKind::Percent,
            "100",
            &["PrintConfig.hpp:1189", "PrintConfig.cpp:1237"][..],
        ),
        (
            "internal_bridge_density",
            OptionValueKind::Percent,
            "100",
            &["PrintConfig.hpp:991", "PrintConfig.cpp:1252"][..],
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
