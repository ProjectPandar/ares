//! `reorder_filaments_for_minimum_flush_volume_with_nozzle_map` (`ToolOrderUtils.cpp:1324–1494`).

use super::Context;
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn sequences(context: &Context, map: &[usize]) -> (i32, Vec<Vec<usize>>) {
    let mut by_extruder = BTreeMap::<usize, Vec<usize>>::new();
    let mut fragments = vec![Vec::new(); context.nozzle_groups.len()];
    let mut flush = 0;
    for nozzle in context
        .used
        .iter()
        .map(|&id| map[id])
        .collect::<BTreeSet<_>>()
    {
        let head = context.nozzle_groups[nozzle].extruder;
        by_extruder.entry(head).or_default().push(nozzle);
        let (cost, layers) = super::score_nozzle(context, map, nozzle);
        flush += cost;
        fragments[nozzle] = layers;
    }
    let extruders = by_extruder.keys().copied().collect::<Vec<_>>();
    if extruders.is_empty() {
        return (flush, context.layers.clone());
    }
    let mut previous_extruder = 0;
    let mut previous_nozzle = [0; 2];
    let mut sequences = Vec::with_capacity(context.layers.len());
    for (layer, _) in context.layers.iter().enumerate() {
        let mut sequence = Vec::new();
        for step in 0..extruders.len() {
            let head = extruders[(previous_extruder + step) % extruders.len()];
            let nozzles = &by_extruder[&head];
            let start = nozzles
                .iter()
                .position(|&id| id == previous_nozzle[head])
                .unwrap_or(0);
            for nozzle in (0..nozzles.len())
                .map(|offset| nozzles[(start + offset) % nozzles.len()])
                .filter(|&nozzle| !fragments[nozzle][layer].is_empty())
            {
                sequence.extend_from_slice(&fragments[nozzle][layer]);
                previous_nozzle[head] = nozzle;
            }
        }
        if let Some(&last) = sequence.last() {
            previous_extruder = context.nozzle_groups[map[last]].extruder;
        }
        sequences.push(sequence);
    }
    (flush, sequences)
}
