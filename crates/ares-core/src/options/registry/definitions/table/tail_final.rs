use super::{OptionDefinition, OptionValueKind};

pub(super) const TAIL_FINAL_OPTION_DEFINITIONS: &[OptionDefinition] = &[
    definition!(
        "solid_infill_direction",
        Float,
        "45",
        "PrintConfig.hpp:1096; PrintConfig.cpp:2871-2879",
    ),
    definition!(
        "solid_infill_filament",
        Int,
        "1",
        "PrintConfig.hpp:1161; PrintConfig.cpp:5648-5655",
    ),
    definition!(
        "solid_infill_rotate_template",
        String,
        "",
        "PrintConfig.hpp:1097; PrintConfig.cpp:3886-3896",
    ),
    definition!(
        "sparse_infill_acceleration",
        FloatOrPercent,
        "100%",
        "PrintConfig.hpp:1049; PrintConfig.cpp:3114-3122",
    ),
    definition!(
        "sparse_infill_density",
        Percent,
        "20",
        "PrintConfig.hpp:1101; PrintConfig.cpp:2881-2889",
    ),
    definition!(
        "sparse_infill_filament",
        Int,
        "1",
        "PrintConfig.hpp:1121; PrintConfig.cpp:4007-4014",
    ),
    definition!(
        "sparse_infill_flow_ratio",
        Float,
        "1",
        "PrintConfig.hpp:1219; PrintConfig.cpp:1354-1363",
    ),
    definition!(
        "sparse_infill_line_width",
        FloatOrPercent,
        "0",
        "PrintConfig.hpp:1122; PrintConfig.cpp:4016-4026",
    ),
    definition!(
        "sparse_infill_pattern",
        Enum,
        "crosshatch",
        "PrintConfig.hpp:1102; PrintConfig.cpp:2928-2985",
    ),
    definition!(
        "sparse_infill_rotate_template",
        String,
        "",
        "PrintConfig.hpp:1100; PrintConfig.cpp:3872-3884",
    ),
    definition!(
        "sparse_infill_speed",
        Float,
        "100",
        "PrintConfig.hpp:1125; PrintConfig.cpp:4054-4061",
    ),
    definition!(
        "spiral_finishing_flow_ratio",
        Float,
        "0",
        "PrintConfig.hpp:1563; PrintConfig.cpp:5717-5726",
    ),
    definition!(
        "spiral_mode",
        Bool,
        "false",
        "PrintConfig.hpp:1560; PrintConfig.cpp:5678-5684",
    ),
    definition!(
        "spiral_mode_max_xy_smoothing",
        FloatOrPercent,
        "200%",
        "PrintConfig.hpp:1562; PrintConfig.cpp:5693-5704",
    ),
    definition!(
        "spiral_mode_smooth",
        Bool,
        "false",
        "PrintConfig.hpp:1561; PrintConfig.cpp:5686-5691",
    ),
    definition!(
        "spiral_starting_flow_ratio",
        Float,
        "0",
        "PrintConfig.hpp:1564; PrintConfig.cpp:5706-5715",
    ),
];
