use crate::project_slice::prepare_infill::{
    horizontal_shell_promotion::types::PreparedPostHorizontalShellPromotion,
    vertical_shell_assignment::{self, PreparedPostVerticalShellAssignment},
};

pub(super) fn successor(prepared: PreparedPostHorizontalShellPromotion) {
    let PreparedPostHorizontalShellPromotion {
        predecessor,
        objects,
        caches,
        projections,
        trims,
        regularizations,
        filters,
    } = prepared;
    vertical_shell_assignment::dispose(PreparedPostVerticalShellAssignment {
        predecessor,
        objects,
        caches,
        projections,
        trims,
        regularizations,
        filters,
    });
}
