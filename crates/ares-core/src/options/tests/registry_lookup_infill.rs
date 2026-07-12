use crate::{OptionValueKind, option_definition};

#[test]
fn infill_combination_and_rotation_lookup_returns_upstream_metadata() {
    for (key, kind, default_value, source_fragments) in [
        (
            "infill_combination",
            OptionValueKind::Bool,
            "false",
            &["PrintConfig.hpp:1132", "PrintConfig.cpp:3853-3860"][..],
        ),
        (
            "infill_shift_step",
            OptionValueKind::Float,
            "0.4",
            &["PrintConfig.hpp:1099", "PrintConfig.cpp:3862-3870"][..],
        ),
        (
            "infill_wall_overlap",
            OptionValueKind::Percent,
            "15",
            &["PrintConfig.hpp:1123", "PrintConfig.cpp:4028-4039"][..],
        ),
        (
            "sparse_infill_filament",
            OptionValueKind::Int,
            "1",
            &["PrintConfig.hpp:1121", "PrintConfig.cpp:4007-4014"][..],
        ),
        (
            "sparse_infill_line_width",
            OptionValueKind::FloatOrPercent,
            "0",
            &["PrintConfig.hpp:1122", "PrintConfig.cpp:4016-4026"][..],
        ),
        (
            "sparse_infill_speed",
            OptionValueKind::Float,
            "100",
            &["PrintConfig.hpp:1125", "PrintConfig.cpp:4054-4061"][..],
        ),
        (
            "top_bottom_infill_wall_overlap",
            OptionValueKind::Percent,
            "25",
            &["PrintConfig.hpp:1124", "PrintConfig.cpp:4041-4052"][..],
        ),
        (
            "infill_combination_max_layer_height",
            OptionValueKind::FloatOrPercent,
            "100%",
            &["PrintConfig.hpp:1134", "PrintConfig.cpp:3972-3984"][..],
        ),
        (
            "infill_lock_depth",
            OptionValueKind::Float,
            "1",
            &["PrintConfig.hpp:1128", "PrintConfig.cpp:3934-3942"][..],
        ),
        (
            "skeleton_infill_density",
            OptionValueKind::Percent,
            "25",
            &["PrintConfig.hpp:1126", "PrintConfig.cpp:3898-3909"][..],
        ),
        (
            "skeleton_infill_line_width",
            OptionValueKind::FloatOrPercent,
            "100%",
            &["PrintConfig.hpp:1131", "PrintConfig.cpp:3954-3962"][..],
        ),
        (
            "skin_infill_density",
            OptionValueKind::Percent,
            "25",
            &["PrintConfig.hpp:1127", "PrintConfig.cpp:3911-3922"][..],
        ),
        (
            "skin_infill_depth",
            OptionValueKind::Float,
            "2",
            &["PrintConfig.hpp:1129", "PrintConfig.cpp:3924-3932"][..],
        ),
        (
            "skin_infill_line_width",
            OptionValueKind::FloatOrPercent,
            "100%",
            &["PrintConfig.hpp:1130", "PrintConfig.cpp:3944-3952"][..],
        ),
        (
            "symmetric_infill_y_axis",
            OptionValueKind::Bool,
            "false",
            &["PrintConfig.hpp:1098", "PrintConfig.cpp:3964-3970"][..],
        ),
        (
            "solid_infill_rotate_template",
            OptionValueKind::String,
            "",
            &["PrintConfig.hpp:1097", "PrintConfig.cpp:3886-3896"][..],
        ),
        (
            "sparse_infill_rotate_template",
            OptionValueKind::String,
            "",
            &["PrintConfig.hpp:1100", "PrintConfig.cpp:3872-3884"][..],
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
