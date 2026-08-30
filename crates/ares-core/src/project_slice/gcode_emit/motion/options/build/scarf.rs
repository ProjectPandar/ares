use super::super::ScarfOptions;
use crate::{ProcessRegionSourceOptions, RegionOptions};

pub(super) fn from_region(
    default: &ProcessRegionSourceOptions,
    region: Option<&RegionOptions>,
) -> ScarfOptions {
    ScarfOptions {
        seam_slope_type: region.map_or(default.seam_slope_type, |value| value.seam_slope_type),
        conditional: region.map_or(default.seam_slope_conditional.0, |value| {
            value.seam_slope_conditional.0
        }),
        start_height: Some(region.map_or(default.seam_slope_start_height, |value| {
            value.seam_slope_start_height
        })),
        entire_loop: region.map_or(default.seam_slope_entire_loop.0, |value| {
            value.seam_slope_entire_loop.0
        }),
        min_length: region.map_or(default.seam_slope_min_length.0, |value| {
            value.seam_slope_min_length.0
        }),
        steps: region
            .map_or(default.seam_slope_steps.0, |value| value.seam_slope_steps.0)
            .max(0) as usize,
        inner_walls: region.map_or(default.seam_slope_inner_walls.0, |value| {
            value.seam_slope_inner_walls.0
        }),
        speed: Some(region.map_or(default.scarf_joint_speed, |value| value.scarf_joint_speed)),
        flow_ratio: region.map_or(default.scarf_joint_flow_ratio.0, |value| {
            value.scarf_joint_flow_ratio.0
        }),
    }
}
