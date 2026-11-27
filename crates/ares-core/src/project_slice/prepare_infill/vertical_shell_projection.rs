mod gather;
mod stage;

pub(in crate::project_slice) mod types;

use crate::{CancelToken, SliceError, project_slice::prepare_infill::vertical_shells};

pub(in crate::project_slice) use types::PreparedPostVerticalShellProjection;

pub(in crate::project_slice) fn prepare(
    prepared: vertical_shells::PreparedPostVerticalShellCache,
    cancel: &CancelToken,
) -> Result<PreparedPostVerticalShellProjection, SliceError> {
    let projections = stage::project(&prepared, cancel)?;
    let vertical_shells::PreparedPostVerticalShellCache {
        predecessor,
        objects,
        caches,
    } = prepared;
    Ok(PreparedPostVerticalShellProjection {
        predecessor,
        objects,
        caches,
        projections,
    })
}
