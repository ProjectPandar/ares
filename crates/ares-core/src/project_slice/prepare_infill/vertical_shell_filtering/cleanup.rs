use crate::project_slice::prepare_infill::{
    vertical_shell_filtering::types::{
        PreparedPostVerticalShellFiltering, VerticalShellTinyFilter,
    },
    vertical_shell_regularization::{self, PreparedPostVerticalShellRegularization},
};

pub(super) fn predecessor(prepared: PreparedPostVerticalShellRegularization) {
    vertical_shell_regularization::dispose(prepared);
}

pub(super) fn successor(prepared: PreparedPostVerticalShellFiltering) {
    let PreparedPostVerticalShellFiltering {
        predecessor,
        objects,
        caches,
        projections,
        trims,
        regularizations,
        filters,
    } = prepared;
    for object in filters {
        for filter in object.records.into_iter().flatten() {
            let VerticalShellTinyFilter { filtered_shell } = filter;
            drop(filtered_shell);
        }
    }
    vertical_shell_regularization::dispose(PreparedPostVerticalShellRegularization {
        predecessor,
        objects,
        caches,
        projections,
        trims,
        regularizations,
    });
}
