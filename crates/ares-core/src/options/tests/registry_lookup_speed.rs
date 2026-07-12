#[test]
fn exposes_speed_option_definition_lookup() {
    for (key, kind, default_value, source_fragments) in [
        (
            "accel_to_decel_enable",
            crate::OptionValueKind::Bool,
            "true",
            &["PrintConfig.hpp:1419", "PrintConfig.cpp:3152-3157"][..],
        ),
        (
            "accel_to_decel_factor",
            crate::OptionValueKind::Percent,
            "50",
            &["PrintConfig.hpp:1420", "PrintConfig.cpp:3159-3167"][..],
        ),
        (
            "bridge_acceleration",
            crate::OptionValueKind::FloatOrPercent,
            "50%",
            &["PrintConfig.hpp:1047", "PrintConfig.cpp:3104-3112"][..],
        ),
        (
            "default_jerk",
            crate::OptionValueKind::Float,
            "0",
            &["PrintConfig.hpp:1052", "PrintConfig.cpp:3169-3176"][..],
        ),
        (
            "default_junction_deviation",
            crate::OptionValueKind::Float,
            "0",
            &["PrintConfig.hpp:1060", "PrintConfig.cpp:3178-3186"][..],
        ),
        (
            "infill_jerk",
            crate::OptionValueKind::Float,
            "9",
            &["PrintConfig.hpp:1055", "PrintConfig.cpp:3215-3222"][..],
        ),
        (
            "initial_layer_acceleration",
            crate::OptionValueKind::Float,
            "300",
            &["PrintConfig.hpp:1046", "PrintConfig.cpp:3134-3141"][..],
        ),
        (
            "initial_layer_infill_speed",
            crate::OptionValueKind::Float,
            "60",
            &["PrintConfig.hpp:1532", "PrintConfig.cpp:3288-3294"][..],
        ),
        (
            "initial_layer_jerk",
            crate::OptionValueKind::Float,
            "9",
            &["PrintConfig.hpp:1057", "PrintConfig.cpp:3224-3231"][..],
        ),
        (
            "initial_layer_line_width",
            crate::OptionValueKind::FloatOrPercent,
            "0",
            &["PrintConfig.hpp:1527", "PrintConfig.cpp:3251-3261"][..],
        ),
        (
            "initial_layer_print_height",
            crate::OptionValueKind::Float,
            "0.2",
            &["PrintConfig.hpp:1528", "PrintConfig.cpp:3264-3270"][..],
        ),
        (
            "initial_layer_speed",
            crate::OptionValueKind::Float,
            "30",
            &["PrintConfig.hpp:1529", "PrintConfig.cpp:3280-3286"][..],
        ),
        (
            "initial_layer_travel_acceleration",
            crate::OptionValueKind::FloatOrPercent,
            "100%",
            &["PrintConfig.hpp:1422", "PrintConfig.cpp:3143-3150"][..],
        ),
        (
            "initial_layer_travel_jerk",
            crate::OptionValueKind::FloatOrPercent,
            "100%",
            &["PrintConfig.hpp:1423", "PrintConfig.cpp:3242-3249"][..],
        ),
        (
            "initial_layer_travel_speed",
            crate::OptionValueKind::FloatOrPercent,
            "100%",
            &["PrintConfig.hpp:1421", "PrintConfig.cpp:3296-3304"][..],
        ),
        (
            "inner_wall_acceleration",
            crate::OptionValueKind::Float,
            "10000",
            &["PrintConfig.hpp:1044", "PrintConfig.cpp:3068-3075"][..],
        ),
        (
            "inner_wall_jerk",
            crate::OptionValueKind::Float,
            "9",
            &["PrintConfig.hpp:1054", "PrintConfig.cpp:3197-3204"][..],
        ),
        (
            "internal_solid_infill_acceleration",
            crate::OptionValueKind::FloatOrPercent,
            "100%",
            &["PrintConfig.hpp:1050", "PrintConfig.cpp:3124-3132"][..],
        ),
        (
            "outer_wall_acceleration",
            crate::OptionValueKind::Float,
            "500",
            &["PrintConfig.hpp:1043", "PrintConfig.cpp:3095-3102"][..],
        ),
        (
            "outer_wall_jerk",
            crate::OptionValueKind::Float,
            "9",
            &["PrintConfig.hpp:1053", "PrintConfig.cpp:3188-3195"][..],
        ),
        (
            "pellet_flow_coefficient",
            crate::OptionValueKind::Floats,
            "0.4157",
            &["PrintConfig.cpp:2551-2555"][..],
        ),
        (
            "sparse_infill_acceleration",
            crate::OptionValueKind::FloatOrPercent,
            "100%",
            &["PrintConfig.hpp:1049", "PrintConfig.cpp:3114-3122"][..],
        ),
        (
            "slow_down_layers",
            crate::OptionValueKind::Int,
            "0",
            &["PrintConfig.hpp:1627", "PrintConfig.cpp:3306-3314"][..],
        ),
        (
            "top_surface_acceleration",
            crate::OptionValueKind::Float,
            "500",
            &["PrintConfig.hpp:1045", "PrintConfig.cpp:3086-3093"][..],
        ),
        (
            "top_surface_jerk",
            crate::OptionValueKind::Float,
            "9",
            &["PrintConfig.hpp:1056", "PrintConfig.cpp:3206-3213"][..],
        ),
        (
            "travel_acceleration",
            crate::OptionValueKind::Float,
            "10000",
            &["PrintConfig.hpp:1048", "PrintConfig.cpp:3077-3084"][..],
        ),
        (
            "travel_jerk",
            crate::OptionValueKind::Float,
            "12",
            &["PrintConfig.hpp:1058", "PrintConfig.cpp:3233-3240"][..],
        ),
        (
            "volumetric_speed_coefficients",
            crate::OptionValueKind::Strings,
            "",
            &["PrintConfig.hpp:1319", "PrintConfig.cpp:2567-2569"][..],
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
