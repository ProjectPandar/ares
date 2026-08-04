use crate::project_slice::prepare_infill::{
    vertical_shell_projection::{self, PreparedPostVerticalShellProjection},
    vertical_shell_trimming::types::{PreparedPostVerticalShellTrim, VerticalShellTrim},
};

pub(super) fn predecessor(prepared: PreparedPostVerticalShellProjection) {
    vertical_shell_projection::dispose(prepared);
}

pub(super) fn successor(prepared: PreparedPostVerticalShellTrim) {
    let PreparedPostVerticalShellTrim {
        predecessor,
        objects,
        caches,
        projections,
        trims,
    } = prepared;
    for object in trims {
        for trim in object.records.into_iter().flatten() {
            let VerticalShellTrim { shell } = trim;
            drop(shell);
        }
    }
    vertical_shell_projection::dispose(PreparedPostVerticalShellProjection {
        predecessor,
        objects,
        caches,
        projections,
    });
}
