mod cache;
mod cleanup;
mod physical_cache;
mod stage;

pub(in crate::project_slice) mod types;

pub(in crate::project_slice) use types::PreparedPostVerticalShellCache;

use crate::{
    SliceError, project_slice::prepare_infill::fill_surfaces::PreparedPostFillSurfacePreparation,
};

pub(in crate::project_slice) fn prepare(
    prepared: PreparedPostFillSurfacePreparation,
) -> Result<PreparedPostVerticalShellCache, SliceError> {
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
