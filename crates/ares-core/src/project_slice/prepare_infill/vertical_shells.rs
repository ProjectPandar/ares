mod cache;
mod physical_cache;
mod stage;

pub(in crate::project_slice) mod types;

pub(in crate::project_slice) use types::PreparedPostVerticalShellCache;

use crate::{
    CancelToken, SliceError,
    project_slice::prepare_infill::fill_surfaces::PreparedPostFillSurfacePreparation,
};

pub(in crate::project_slice) fn prepare(
    prepared: PreparedPostFillSurfacePreparation,
    cancel: &CancelToken,
) -> Result<PreparedPostVerticalShellCache, SliceError> {
    let caches = stage::project(&prepared, cancel)?;
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
