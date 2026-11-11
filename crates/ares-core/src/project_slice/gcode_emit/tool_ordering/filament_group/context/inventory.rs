use super::{DynamicPrintConfig, NozzleGroup, SliceError, unsupported};
use crate::project_slice::gcode_emit::tool_ordering::nozzle_group::canonical_diameter;
use std::collections::BTreeMap;

// ToolOrdering.cpp:1503-1527: expand each extruder into physical nozzle groups.
pub(super) fn build_nozzle_groups(
    counts: &[usize],
    volumes: &[i32; 2],
    diameters: &crate::OrcaFloats,
) -> Vec<NozzleGroup> {
    let mut groups = Vec::new();
    let mut group_id = 0;
    for (extruder, count) in counts.iter().copied().enumerate() {
        let diameter = canonical_diameter(diameters.0[extruder].0);
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
        let mut stats = BTreeMap::new();
        for token in wire.split('|') {
            let mut parts = token.split('#');
            let volume = match parts.next().unwrap() {
                "Standard" => 0,
                "High Flow" => 1,
                "Hybrid" => 2,
                "TPU High Flow" => 3,
                "E3D High Flow" => 5,
                "Extra High Flow" => 6,
                _ => continue,
            };
            let Some(count) = parts.next() else { continue };
            stats.insert(volume, atoi(count));
        }
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

// std::atoi consumes a signed C-locale decimal prefix (PrintConfig.cpp:967).
fn atoi(text: &str) -> i32 {
    let text = text.trim_start_matches([' ', '\t', '\n', '\u{b}', '\u{c}', '\r']);
    let sign = usize::from(text.starts_with(['+', '-']));
    let end = sign
        + text
            .bytes()
            .skip(sign)
            .take_while(u8::is_ascii_digit)
            .count();
    text[..end].parse().unwrap_or(0)
}
