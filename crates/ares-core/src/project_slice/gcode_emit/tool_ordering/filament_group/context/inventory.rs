use super::{DynamicPrintConfig, NozzleGroup, SliceError, unsupported};
use crate::project_slice::gcode_emit::tool_ordering::nozzle_group::{
    canonical_diameter, parse_nozzle_counts,
};

// ToolOrdering.cpp:1503-1527: expand each extruder into physical nozzle groups.
pub(super) fn build_nozzle_groups(
    counts: &[usize],
    volumes: &[i32; 2],
    diameters: &crate::OrcaFloats,
) -> Vec<NozzleGroup> {
    let mut groups = Vec::new();
    let mut group_id = 0;
    for (extruder, diameter) in diameters.0.iter().enumerate() {
        let count = counts.get(extruder).copied().unwrap_or(1);
        let diameter = canonical_diameter(diameter.0);
        for _ in 0..count {
            groups.push(NozzleGroup {
                extruder,
                group_id,
                diameter,
                volume: volumes[extruder],
            });
            group_id += 1;
        }
    }
    groups
}

pub(super) fn check_nozzle_stats(
    config: &DynamicPrintConfig,
    volumes: &[i32; 2],
) -> Result<(), SliceError> {
    // PrintConfig.cpp:940–972 / ToolOrdering.cpp:1508–1530. One physical
    // nozzle of one volume type is the admitted inventory boundary.
    for (head, wire) in config
        .option("extruder_nozzle_stats")
        .strings()
        .iter()
        .enumerate()
        .take(2)
    {
        let stats = parse_nozzle_counts(wire);
        if !stats.is_empty() {
            if stats.len() != 1 || stats.values().any(|v| *v != 1) || stats.contains_key(&2) {
                return Err(unsupported());
            }
            // Non-hybrid build_nozzle_groups retains the configured volume.
            if !stats.contains_key(&volumes[head]) {
                return Err(unsupported());
            }
        }
    }
    Ok(())
}
