mod cache;
mod cleanup;
mod stage;
#[cfg(test)]
mod tests;
pub(in crate::project_slice) mod types;

#[cfg(test)]
pub(in crate::project_slice) use cache::{
    GeometryStep, fail_geometry_at, geometry_events, reset_geometry_hooks,
};
pub(in crate::project_slice) use types::PreparedPostVerticalShellCache;

use crate::{
    SliceError, project_slice::prepare_infill::fill_surfaces::PreparedPostFillSurfacePreparation,
};

#[cfg(test)]
thread_local! {
    static INVOCATIONS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

pub(in crate::project_slice) fn prepare(
    prepared: PreparedPostFillSurfacePreparation,
) -> Result<PreparedPostVerticalShellCache, SliceError> {
    #[cfg(test)]
    INVOCATIONS.with(|count| count.set(count.get() + 1));

    let caches = match stage::project(&prepared) {
        Ok(caches) => caches,
        Err(error) => {
            cleanup::predecessor(prepared);
            return Err(error);
        }
    };
    let PreparedPostFillSurfacePreparation {
        predecessor,
        objects,
    } = prepared;
    Ok(PreparedPostVerticalShellCache {
        predecessor,
        objects,
        caches,
    })
}

pub(in crate::project_slice) fn dispose(prepared: PreparedPostVerticalShellCache) {
    cleanup::successor(prepared);
}

#[cfg(test)]
pub(in crate::project_slice) fn invocations() -> usize {
    INVOCATIONS.with(std::cell::Cell::get)
}

#[cfg(test)]
pub(in crate::project_slice) fn reset_invocations() {
    INVOCATIONS.with(|count| count.set(0));
}
