use crate::project_slice::gcode_emit::wipe_tower_estimate::config::{
    ConfigValue, DynamicPrintConfig,
};
use crate::project_slice::perimeters::classic::traversal::PreparedPostClassicTraversal;
use crate::{NozzleVolumeType, ProjectSettings};

// ToolOrdering.cpp:1503-1527: the physical groups retained by the grouping result.
pub(in crate::project_slice::gcode_emit) struct NozzleGroup {
    pub(in crate::project_slice::gcode_emit) extruder: usize,
    pub(in crate::project_slice::gcode_emit) group_id: usize,
    pub(in crate::project_slice::gcode_emit) diameter: f64,
    pub(in crate::project_slice::gcode_emit) volume: i32,
}

// MultiNozzleUtils.cpp:319-369: one physical group per head on the static path.
pub(super) fn static_groups(settings: &ProjectSettings) -> Vec<NozzleGroup> {
    settings
        .project
        .print
        .nozzle_diameter
        .0
        .iter()
        .enumerate()
        .map(|(extruder, diameter)| NozzleGroup {
            extruder,
            group_id: extruder,
            diameter: canonical_diameter(diameter.0),
            volume: settings
                .project
                .gcode
                .nozzle_volume_type
                .0
                .get(extruder)
                .copied()
                .unwrap_or(NozzleVolumeType::Standard) as i32,
        })
        .collect()
}

// utils.cpp:1784-1791: strict nearest supported diameter; first wins ties.
pub(super) fn canonical_diameter(diameter: f64) -> f64 {
    [0.4, 0.6, 0.8].into_iter().fold(0.2, |closest, candidate| {
        if (candidate - diameter).abs() < (closest - diameter).abs() {
            candidate
        } else {
            closest
        }
    })
}

pub(in crate::project_slice::gcode_emit) fn nozzle_count(
    traversal: &PreparedPostClassicTraversal,
) -> Vec<i64> {
    let mut config = DynamicPrintConfig::full_print_config();
    if let Some(token) = traversal
        .project
        .documents()
        .project_settings_raw
        .iter()
        .find_map(|entry| match entry {
            Ok(("extruder_max_nozzle_count", Some(token))) => Some(token),
            _ => None,
        })
    {
        config
            .set_deserialize_strict("extruder_max_nozzle_count", &token)
            .expect("validated extruder nozzle count");
    }
    let ConfigValue::Ints(counts) = config.option("extruder_max_nozzle_count") else {
        unreachable!("extruder_max_nozzle_count is an integer vector")
    };
    counts
}
