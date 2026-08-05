use crate::project_slice::prepare_infill::{
    vertical_shell_assignment::types::PreparedPostVerticalShellAssignment,
    vertical_shell_filtering::{self, PreparedPostVerticalShellFiltering},
};

pub(super) fn successor(prepared: PreparedPostVerticalShellAssignment) {
    let PreparedPostVerticalShellAssignment {
        predecessor,
        objects,
        caches,
        projections,
        trims,
        regularizations,
        filters,
    } = prepared;
    vertical_shell_filtering::dispose(PreparedPostVerticalShellFiltering {
        predecessor,
        objects,
        caches,
        projections,
        trims,
        regularizations,
        filters,
    });
}
