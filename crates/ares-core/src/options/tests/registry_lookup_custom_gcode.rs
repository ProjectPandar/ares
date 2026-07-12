use crate::{OptionValueKind, option_definition};

#[test]
fn custom_gcode_and_small_area_lookup_returns_upstream_metadata() {
    for (key, kind, default_value, source_fragments) in [
        (
            "layer_change_gcode",
            OptionValueKind::String,
            "",
            &["PrintConfig.hpp:1358", "PrintConfig.cpp:4295-4302"][..],
        ),
        (
            "time_lapse_gcode",
            OptionValueKind::String,
            "",
            &["PrintConfig.hpp:1359", "PrintConfig.cpp:4304-4310"][..],
        ),
        (
            "wrapping_detection_gcode",
            OptionValueKind::String,
            "",
            &["PrintConfig.hpp:1360", "PrintConfig.cpp:4312-4318"][..],
        ),
        (
            "silent_mode",
            OptionValueKind::Bool,
            "false",
            &["PrintConfig.hpp:1398", "PrintConfig.cpp:4320-4324"][..],
        ),
        (
            "emit_machine_limits_to_gcode",
            OptionValueKind::Bool,
            "true",
            &["PrintConfig.hpp:1247", "PrintConfig.cpp:4326-4332"][..],
        ),
        (
            "machine_pause_gcode",
            OptionValueKind::String,
            "",
            &["PrintConfig.hpp:1399", "PrintConfig.cpp:4334-4341"][..],
        ),
        (
            "template_custom_gcode",
            OptionValueKind::String,
            "",
            &["PrintConfig.hpp:1400", "PrintConfig.cpp:4343-4350"][..],
        ),
        (
            "small_area_infill_flow_compensation",
            OptionValueKind::Bool,
            "false",
            &["PrintConfig.hpp:1211", "PrintConfig.cpp:4352-4357"][..],
        ),
        (
            "small_area_infill_flow_compensation_model",
            OptionValueKind::Strings,
            "0,0
0.2,0.4444
0.4,0.6145
0.6,0.7059
0.8,0.7619
1.5,0.8571
2,0.8889
3,0.9231
5,0.9520
10,1",
            &["PrintConfig.hpp:1464", "PrintConfig.cpp:4359-4371"][..],
        ),
        (
            "has_scarf_joint_seam",
            OptionValueKind::Bool,
            "false",
            &["PrintConfig.hpp:1466", "PrintConfig.cpp:4373-4375"][..],
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
