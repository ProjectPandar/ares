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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn priming_ends_on_the_first_printing_tool() {
        let layers = vec![
            LayerTools {
                extruders: vec![0],
                ..Default::default()
            },
            LayerTools {
                extruders: vec![1],
                ..Default::default()
            },
        ];
        assert_eq!(collect(&layers, false), (vec![0, 1], Some(0)));
        assert_eq!(collect(&layers, true), (vec![1, 0], Some(1)));
    }
}
