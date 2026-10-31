//! `ToolOrdering::collect_extruder_statistics` priming order.

use super::LayerTools;

pub(super) fn collect(
    layers: &[LayerTools],
    prime_multi_material: bool,
) -> (Vec<usize>, Option<usize>) {
    let mut first = layers
        .iter()
        .find_map(|layer| layer.extruders.first().copied());
    let mut all = layers
        .iter()
        .flat_map(|layer| layer.extruders.iter().copied())
        .collect::<Vec<_>>();
    all.sort_unstable();
    all.dedup();
    if prime_multi_material && let Some(first_printing) = first {
        all.retain(|&tool| tool != first_printing);
        all.push(first_printing);
        first = all.first().copied();
    }
    (all, first)
}
