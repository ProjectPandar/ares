mod cache;
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
    let caches = stage::project(&prepared)?;
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
