//! Merged-chunk layer bookkeeping: previous-z for `;HEIGHT:`/`change_layer`
//! diffs and the physical boundary layer index (`m_layer_index` /
//! `layer->id()` including raft layers, `GCode.cpp:5339-5344`,
//! `GCode.cpp:5685-5690`).

use super::raft_schedule;
use crate::project_slice::raft::RaftLayerPlan;

/// `previous_layer_z` = the last physical layer's print z (the final raft
/// layer for the first object layer); `boundary_layer_index` = the
/// physical index (`raft_index` for raft chunks, `first_layer +
/// raft_count` for object chunks); `leading_raft` = the raft chunk's
/// `(raft_index, first_object)` when the chunk is raft;
/// `object_layer_over_raft` = `m_object_layer_over_raft`
/// (`GCode.cpp:5339-5344`). Returns `(previous_z, boundary_index,
/// object_layer_over_raft, leading_raft)`.
pub(super) fn chunk_bookkeeping(
    raft: Option<&raft_schedule::RaftSchedule>,
    raft_plans: Option<&Vec<Vec<RaftLayerPlan>>>,
    per_object_z: &[Vec<f64>],
    first_object: usize,
    first_layer: usize,
) -> (f64, usize, bool, Option<(usize, usize)>) {
    // Raft chunks use the raft grid for the previous z and the raft
    // layer index for the CHANGE_LAYER numbering (the raft flange is
    // print layer 0).
    let leading_raft = raft
        .and_then(|_raft| raft_schedule::classify(first_layer).ok().flatten())
        .map(|raft_index| (raft_index, first_object));
    let (previous_layer_z, boundary_layer_index) = match leading_raft {
        Some((raft_index, _)) => {
            let previous = if raft_index == 0 {
                0.0
            } else {
                raft_plans
                    .and_then(|plans| plans.get(first_object))
                    .and_then(|plans| plans.get(raft_index - 1))
                    .map(|plan| plan.z.print_z)
                    .unwrap_or(0.0)
            };
            (previous, raft_index)
        }
        None => {
            // `GCode::change_layer` diffs `print_z - m_last_layer_z`
            // (`GCode.cpp:4630`), where `m_last_layer_z` carries the last
            // PHYSICAL layer's z — including the final raft layer for the
            // first object layer (`;HEIGHT:0.3`, not the full print z).
            let previous = if first_layer > 0 {
                per_object_z[first_object]
                    .get(first_layer - 1)
                    .copied()
                    .unwrap_or(0.0)
            } else {
                raft.and_then(|raft| raft.counts.get(first_object))
                    .and_then(|&count| {
                        raft_plans.and_then(|plans| {
                            plans
                                .get(first_object)?
                                .get(count.checked_sub(1)?)
                                .map(|plan| plan.z.print_z)
                        })
                    })
                    .unwrap_or(0.0)
            };
            // `layer->id()` counts PHYSICAL layers — upstream's raft
            // layers occupy ids `0..raft_count` inside the object's
            // layer vector, so the first object layer is id
            // `raft_count` (`GCode.cpp:5339-5344` keys
            // `object_layer_over_raft` on `layer->id() ==
            // raft_layers`). `m_layer_index` (`GCode.cpp:5689`) counts
            // the same physical index.
            let raft_count = raft
                .and_then(|raft| raft.counts.get(first_object))
                .copied()
                .unwrap_or(0);
            (previous, first_layer + raft_count)
        }
    };

    // `m_object_layer_over_raft` (`GCode.cpp:5339-5344`): the first
    // object layer over a raft — `layer->id() == raft_layers()`.
    let object_layer_over_raft = leading_raft.is_none()
        && first_layer == 0
        && raft
            .and_then(|raft| raft.counts.get(first_object))
            .copied()
            .unwrap_or(0)
            > 0;
    (
        previous_layer_z,
        boundary_layer_index,
        object_layer_over_raft,
        leading_raft,
    )
}
