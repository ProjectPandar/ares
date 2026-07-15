use std::collections::BTreeMap;

use super::graph::{LoadedModel, ModelGraph};

pub(super) fn group_extruders(graph: &ModelGraph) -> BTreeMap<i32, i32> {
    let mut groups = BTreeMap::<i32, String>::new();
    for model in &graph.models[1..] {
        for (id, color) in model_colors(model) {
            groups.entry(id).or_insert(color);
        }
    }
    for (id, color) in model_colors(graph.root()) {
        groups.insert(id, color);
    }

    let mut colors = BTreeMap::<String, i32>::new();
    let mut next_extruder = 1_i32;
    groups
        .into_iter()
        .map(|(group_id, color)| {
            let extruder = *colors.entry(color).or_insert_with(|| {
                let current = next_extruder;
                next_extruder += 1;
                current
            });
            (group_id, extruder)
        })
        .collect()
}

fn model_colors(model: &LoadedModel) -> BTreeMap<i32, String> {
    let mut output = BTreeMap::new();
    for group in &model.document.resources.color_groups {
        if let Some(color) = group.colors.last() {
            output.insert(group.id, color.color.clone());
        }
    }
    output
}
