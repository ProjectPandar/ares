mod detect_bridge_directions;
mod expand_bridges_detect_orientations;
mod expand_expolygons;
mod expand_merge;
mod group_bridges;
pub(in crate::project_slice) mod merge_bridges;
mod parameters;
mod process;
mod stage;
#[cfg(test)]
mod tests;
mod types;

use crate::{
    SliceError,
    project_slice::prepare_infill::horizontal_shell_propagation::PreparedPostHorizontalShellPropagation,
};

pub(in crate::project_slice) use stage::PreparedPostExternalSurfaces;
pub(in crate::project_slice) use types::{Bridge, ExpansionResult, ExpansionZone};

pub(in crate::project_slice) fn active_region_ids(
    prepared: &PreparedPostExternalSurfaces,
    object_index: usize,
) -> Vec<usize> {
    let horizontal = &prepared.predecessor;
    let traversal = &horizontal.predecessor;
    let traversal_object = &traversal.objects[object_index];
    let prelude = &traversal_object
        .predecessor
        .predecessor
        .predecessor
        .predecessor
        .object;
    let (post_regions, _) = prelude.as_parts().0.as_parts();
    let (plan, _, regions) = post_regions.as_parts();
    regions
        .iter()
        .enumerate()
        .filter_map(|(region_index, _)| {
            (0..plan.layers.len())
                .any(|layer_index| {
                    let slot = prelude.region_layer_record_index(
                        crate::project_slice::perimeters::types::RegionLayerIndex {
                            region_index,
                            layer_index,
                        },
                    );
                    horizontal.objects[object_index].records[slot]
                        .as_ref()
                        .is_some_and(|record| !record.fill_surfaces.is_empty())
                })
                .then_some(region_index)
        })
        .collect()
}

#[cfg(test)]
thread_local! {
    static INVOCATIONS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    static DISPOSALS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

pub(in crate::project_slice) fn prepare(
    predecessor: PreparedPostHorizontalShellPropagation,
) -> Result<PreparedPostExternalSurfaces, SliceError> {
    #[cfg(test)]
    INVOCATIONS.with(|count| count.set(count.get() + 1));
    stage::prepare(predecessor)
}

pub(in crate::project_slice) fn dispose(prepared: PreparedPostExternalSurfaces) {
    #[cfg(test)]
    DISPOSALS.with(|count| count.set(count.get() + 1));
    stage::dispose(prepared);
}

#[cfg(test)]
pub(in crate::project_slice) fn reset_hooks() {
    INVOCATIONS.with(|count| count.set(0));
    DISPOSALS.with(|count| count.set(0));
}

#[cfg(test)]
pub(in crate::project_slice) fn invocations() -> usize {
    INVOCATIONS.with(std::cell::Cell::get)
}

#[cfg(test)]
pub(in crate::project_slice) fn disposals() -> usize {
    DISPOSALS.with(std::cell::Cell::get)
}
