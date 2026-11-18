//! `WipeTower.cpp:3808–3862,3912–3934`: wall material ownership.
use super::plan::{Layer, Plan};
use std::collections::{BTreeMap, BTreeSet};

fn count_layer(
    layer: &Layer,
    ids: &[i32],
    categories: &mut BTreeMap<i32, usize>,
    filaments: &mut BTreeMap<usize, usize>,
) {
    let first = layer.changes[0].old;
    let mut used_tools = BTreeSet::from([first]);
    let mut used_categories = BTreeSet::from([ids[first]]);
    for change in &layer.changes {
        if used_categories.insert(ids[change.new]) {
            *categories.entry(ids[change.new]).or_default() += 1;
        }
        if used_tools.insert(change.new) {
            *filaments.entry(change.new).or_default() += 1;
        }
    }
}

impl Plan {
    pub(super) fn wall_filament(&self, mut current: usize) -> usize {
        let mut categories = BTreeMap::<i32, usize>::new();
        let mut filaments = BTreeMap::<usize, usize>::new();
        for layer in &self.layers {
            if layer.changes.is_empty() {
                *filaments.entry(current).or_default() += 1;
                *categories.entry(self.categories[current]).or_default() += 1;
                continue;
            }
            let first = &layer.changes[0];
            *filaments.entry(first.old).or_default() += 1;
            *categories.entry(self.categories[first.old]).or_default() += 1;
            count_layer(layer, &self.categories, &mut categories, &mut filaments);
            current = layer.changes.last().unwrap().new;
        }
        // Strict greater-than over source std::map order retains the lowest key on ties.
        let mut selected = None;
        let mut count = 0;
        for (category, frequency) in categories {
            if frequency > count {
                selected = Some(category);
                count = frequency;
            }
        }
        let category = selected.expect("nonempty tower plan has a wall category");
        let mut selected = None;
        count = 0;
        for (filament, frequency) in filaments {
            if self.categories[filament] == category && frequency > count {
                selected = Some(filament);
                count = frequency;
            }
        }
        selected.expect("wall category retains its filament")
    }

    pub(super) fn layer_wall_filament(&self, index: usize, wall: usize) -> Option<usize> {
        let layer = &self.layers[index];
        let first = layer.changes.first()?;
        let mut candidate = None;
        if self.is_valid_last_layer(first.old, index, layer.z) {
            if first.old == wall {
                return Some(wall);
            }
            if self.categories[first.old] == self.categories[wall] {
                candidate = Some(first.old);
            }
        }
        for change in &layer.changes {
            if change.new == wall {
                return Some(wall);
            }
            if candidate.is_none() && self.categories[change.new] == self.categories[wall] {
                candidate = Some(change.new);
            }
        }
        Some(candidate.unwrap_or(first.new))
    }
}
