//! `TreeSupport3D.cpp::merge_influence_areas`.

mod compaction;
mod ordering;

use super::super::model_volumes::TreeModelVolumes;
use super::{TreeSettings, layer_growth::SupportElementMerging};
use crate::geometry::{ClipperError, CoordinateScale};

pub(super) fn merge_influence_areas(
    volumes: &mut TreeModelVolumes<'_>,
    config: &TreeSettings,
    layer: usize,
    nodes: &mut [SupportElementMerging],
    scale: CoordinateScale,
) -> Result<(), ClipperError> {
    let input_size = nodes.len();
    if input_size == 0 {
        return Ok(());
    }
    ordering::build_recursive(nodes);
    // TreeSupport3D.cpp:2386-2391 selects the initial bucket layout from the
    // runtime TBB arena concurrency. Ares deliberately fixes that input to a
    // single worker (num_threads = 1) so the layout never depends on the
    // machine; see docs/architecture/2026-10-14-organic-merge-single-worker-spec.md
    // superseding ARD-0102's keep-native-parallelism clause.
    let num_buckets_min = (input_size + 2) / 4;
    let num_buckets_max = input_size / 2;
    let (num_buckets_initial, bucket_size) = if num_buckets_min >= 1 {
        (num_buckets_min, 4)
    } else {
        (num_buckets_max, 2)
    };
    let mut buckets = Vec::with_capacity(num_buckets_initial + 1);
    let mut start = 0;
    for _ in 0..num_buckets_initial {
        buckets.push(start..start + bucket_size);
        start += bucket_size;
    }
    let last = buckets
        .last_mut()
        .expect("native pathing merges multiple nodes");
    if last.end >= input_size {
        last.end = last.end.min(input_size);
    } else {
        debug_assert_eq!(start + 1, input_size);
        buckets.push(start..input_size);
    }
    for bucket in &mut buckets[..num_buckets_initial] {
        bucket.end =
            compaction::merge_leaves(volumes, config, (layer, scale), nodes, bucket.clone())?;
    }
    while buckets.len() > 1 {
        for index in 0..buckets.len() / 2 {
            let even = index * 2;
            buckets[even].end = compaction::merge_sets(
                volumes,
                config,
                (layer, scale),
                nodes,
                (buckets[even].clone(), buckets[even + 1].clone()),
            )?;
        }
        let new_size = buckets.len().div_ceil(2);
        for index in 1..new_size {
            buckets[index] = buckets[index * 2].clone();
        }
        buckets.truncate(new_size);
    }
    Ok(())
}
