//! `reorder_filaments_for_minimum_flush_volume` sequence-producing path.

use std::collections::{BTreeMap, HashMap};

use super::super::LayerTools;
use super::solve;

pub(super) fn reorder(
    layers: &mut [LayerTools],
    maps: &[usize],
    matrices: &[Vec<Vec<f32>>],
    custom: &BTreeMap<usize, Vec<usize>>,
) {
    let sequences = [
        group_sequences(layers, maps, &matrices[0], custom, 0),
        if maps.contains(&1) {
            group_sequences(layers, maps, &matrices[1], custom, 1)
        } else {
            Vec::new()
        },
    ];
    let mut last_group = 0;
    if let Some((&layer, first)) = custom.first_key_value() {
        let first_group = usize::from(maps[first[0]] != 0);
        last_group = if layer & 1 != 0 {
            1 - first_group
        } else {
            first_group
        };
    }
    for (index, layer) in layers.iter_mut().enumerate() {
        if let Some(sequence) = custom.get(&index) {
            layer.extruders.clone_from(sequence);
            if let Some(&last) = sequence.last() {
                last_group = usize::from(maps[last] != 0);
            }
        } else {
            layer.extruders.clear();
            let other_group = 1 - last_group;
            append_group(&mut layer.extruders, &sequences[last_group], index);
            if append_group(&mut layer.extruders, &sequences[other_group], index) {
                last_group = other_group;
            }
        }
    }
}

fn append_group(target: &mut Vec<usize>, sequences: &[Vec<usize>], layer: usize) -> bool {
    if sequences.is_empty() {
        return false;
    }
    target.extend_from_slice(&sequences[layer]);
    !sequences[layer].is_empty()
}

fn in_group(tools: &[usize], maps: &[usize], group: usize) -> Vec<usize> {
    tools
        .iter()
        .copied()
        .filter(|&id| maps[id] == group)
        .collect()
}

fn group_sequences(
    layers: &[LayerTools],
    maps: &[usize],
    matrix: &[Vec<f32>],
    custom: &BTreeMap<usize, Vec<usize>>,
    group: usize,
) -> Vec<Vec<usize>> {
    if !maps.contains(&group) {
        return Vec::new();
    }
    let mut previous = None;
    let mut cache = HashMap::<u128, Vec<usize>>::new();
    let mut sequences = Vec::with_capacity(layers.len());
    for (index, layer) in layers.iter().enumerate() {
        if let Some(sequence) = custom.get(&index) {
            if let Some(&last) = sequence.iter().rev().find(|&&id| maps[id] == group) {
                previous = Some(last);
            }
            sequences.push(Vec::new());
            continue;
        }
        let current = in_group(&layer.extruders, maps, group);
        let next = layers
            .get(index + 1)
            .map_or_else(Vec::new, |layer| in_group(&layer.extruders, maps, group));
        let forecast = current.len() <= 5 && next.len() <= 5;
        let key = cache_key(&current, &next, previous, forecast);
        let sequence = cache
            .entry(key)
            .or_insert_with(|| solve::order(matrix, &current, &next, previous, forecast));
        if let Some(&last) = sequence.last() {
            previous = Some(last);
        }
        sequences.push(sequence.clone());
    }
    sequences
}

fn cache_key(current: &[usize], next: &[usize], previous: Option<usize>, forecast: bool) -> u128 {
    // boost::multiprecision::uint128_t discards bits beyond the fixed width.
    let bit = |shift: usize| 1u128.checked_shl(shift as u32).unwrap_or(0);
    let mut key = previous.map_or(0, |id| bit(64 + id));
    if forecast {
        for &id in next {
            key |= bit(32 + id);
        }
    }
    for &id in current {
        key |= bit(id);
    }
    key
}
