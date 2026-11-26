mod gather;
mod stage;

pub(in crate::project_slice) mod types;

use crate::{SliceError, project_slice::prepare_infill::vertical_shells};

pub(in crate::project_slice) use types::PreparedPostVerticalShellProjection;

pub(in crate::project_slice) fn prepare(
    prepared: vertical_shells::PreparedPostVerticalShellCache,
) -> Result<PreparedPostVerticalShellProjection, SliceError> {
    let projections = stage::project(&prepared)?;
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
