use crate::{OptionValueKind, option_definition};

#[test]
fn custom_gcode_and_small_area_lookup_preserves_registry_contract() {
    for (key, kind, default_value) in [
        (
            "layer_change_gcode",
            OptionValueKind::String,
            "",
        ),
        (
            "time_lapse_gcode",
            OptionValueKind::String,
            "",
        ),
        (
            "wrapping_detection_gcode",
            OptionValueKind::String,
            "",
        ),
        (
            "silent_mode",
            OptionValueKind::Bool,
            "false",
        ),
        (
            "emit_machine_limits_to_gcode",
            OptionValueKind::Bool,
            "true",
        ),
        (
            "machine_pause_gcode",
            OptionValueKind::String,
            "",
        ),
        (
            "template_custom_gcode",
            OptionValueKind::String,
            "",
        ),
        (
            "small_area_infill_flow_compensation",
            OptionValueKind::Bool,
            "false",
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
        ),
        (
            "has_scarf_joint_seam",
            OptionValueKind::Bool,
            "false",
        ),
    ] {
        let definition = option_definition(key).unwrap();
        assert_eq!(definition.kind, kind);
        assert_eq!(definition.default_value, default_value);
    }
}
