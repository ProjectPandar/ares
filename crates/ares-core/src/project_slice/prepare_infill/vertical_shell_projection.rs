mod cleanup;
mod gather;
mod stage;

pub(in crate::project_slice) mod types;

use crate::{SliceError, project_slice::prepare_infill::vertical_shells};

pub(in crate::project_slice) use types::PreparedPostVerticalShellProjection;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::project_slice) enum GeometryStep {
    TopVisit,
    BottomVisit,
    HoleIntersection,
    ShellUnion,
    TopAnchorOffset,
    TopAnchorIntersection,
    BottomAnchorOffset,
    BottomAnchorIntersection,
}

pub(in crate::project_slice) fn prepare(
    prepared: vertical_shells::PreparedPostVerticalShellCache,
) -> Result<PreparedPostVerticalShellProjection, SliceError> {
    let projections = match stage::project(&prepared) {
        Ok(projections) => projections,
        Err(error) => {
            cleanup::predecessor(prepared);
            return Err(error);
        }
    };
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

pub(in crate::project_slice) fn dispose(prepared: PreparedPostVerticalShellProjection) {
    cleanup::successor(prepared);
}

fn geometry_step(_step: GeometryStep) -> Result<(), SliceError> {
    Ok(())
}
