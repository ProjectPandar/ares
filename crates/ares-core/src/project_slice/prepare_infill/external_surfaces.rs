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
