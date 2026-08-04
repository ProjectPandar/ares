use crate::project_slice::prepare_infill::{
    vertical_shell_projection::types::{
        PreparedPostVerticalShellProjection, VerticalShellProjection,
    },
    vertical_shells::{self, PreparedPostVerticalShellCache},
};

pub(super) fn predecessor(prepared: PreparedPostVerticalShellCache) {
    vertical_shells::dispose(prepared);
}

pub(super) fn successor(prepared: PreparedPostVerticalShellProjection) {
    let PreparedPostVerticalShellProjection {
        predecessor,
        objects,
        caches,
        projections,
    } = prepared;
    for object in projections {
        for projection in object.records.into_iter().flatten() {
            let VerticalShellProjection { shell, holes } = projection;
            drop(shell);
            drop(holes);
        }
    }
    vertical_shells::dispose(PreparedPostVerticalShellCache {
        predecessor,
        objects,
        caches,
    });
}
