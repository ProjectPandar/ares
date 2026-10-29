//! `WipeTower.cpp:2709–2758`: category-local entry gaps, not Type2's empty mask.
use super::generator::Generator;
use std::collections::BTreeMap;

pub(super) fn prepare(generator: &mut Generator<'_>) {
    generator.wall_gaps.clear();
    let layer = &generator.plan.layers[generator.index];
    let mut depths = BTreeMap::<i32, f32>::new();
    for change in &layer.changes {
        let old = generator.plan.categories[change.old];
        let new = generator.plan.categories[change.new];
        let spacing = if generator.plan.tpu && layer.spacing < 2.0 {
            1.0
        } else {
            layer.spacing
        };
        let nozzle_depth = change.nozzle_depth * spacing;
        let Some(block) = generator
            .plan
            .blocks
            .iter()
            .find(|block| block.category == new)
        else {
            continue;
        };
        let wipe_depth = (change.length / (generator.plan.width - 2.0 * generator.plan.perimeter))
            .ceil()
            * generator.plan.perimeter
            * layer.spacing;
        let mut depth = *depths.entry(new).or_insert(block.start);
        let nozzle_change =
            generator.plan.nozzle_map[change.old] != generator.plan.nozzle_map[change.new];
        if nozzle_change && old == new {
            depth += nozzle_depth;
        }
        if nozzle_change && old != new {
            let Some(old_block) = generator
                .plan
                .blocks
                .iter()
                .find(|block| block.category == old)
            else {
                continue;
            };
            *depths.entry(old).or_insert(old_block.start) += nozzle_depth;
        }
        let y = if generator.index % 2 == 1 {
            depth + wipe_depth - layer.spacing * generator.plan.perimeter
        } else {
            depth
        };
        let x = if matches!(generator.index % 4, 1 | 2) {
            generator.plan.width
        } else {
            0.0
        };
        generator.wall_gaps.push([x, y]);
        depths.insert(
            new,
            depth + change.depth - change.nozzle_depth * layer.spacing,
        );
    }
}
