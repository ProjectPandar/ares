use crate::project_slice::prepare_infill::{
    horizontal_shell_promotion::{self, PreparedPostHorizontalShellPromotion},
    horizontal_shell_propagation::types::PreparedPostHorizontalShellPropagation,
};

pub(super) fn successor(prepared: PreparedPostHorizontalShellPropagation) {
    let PreparedPostHorizontalShellPropagation {
        predecessor,
        objects,
        caches,
        projections,
        trims,
        regularizations,
        filters,
    } = prepared;
    horizontal_shell_promotion::dispose(PreparedPostHorizontalShellPromotion {
        predecessor,
        objects,
        caches,
        projections,
        trims,
        regularizations,
        filters,
    });
}
