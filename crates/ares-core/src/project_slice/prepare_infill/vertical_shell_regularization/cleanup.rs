use crate::project_slice::prepare_infill::{
    vertical_shell_regularization::types::{
        PreparedPostVerticalShellRegularization, VerticalShellRegularization,
    },
    vertical_shell_trimming::{self, PreparedPostVerticalShellTrim},
};

pub(super) fn predecessor(prepared: PreparedPostVerticalShellTrim) {
    vertical_shell_trimming::dispose(prepared);
}

pub(super) fn successor(prepared: PreparedPostVerticalShellRegularization) {
    let PreparedPostVerticalShellRegularization {
        predecessor,
        objects,
        caches,
        projections,
        trims,
        regularizations,
    } = prepared;
    for object in regularizations {
        for regularization in object.records.into_iter().flatten() {
            let VerticalShellRegularization { regularized_shell } = regularization;
            drop(regularized_shell);
        }
    }
    vertical_shell_trimming::dispose(PreparedPostVerticalShellTrim {
        predecessor,
        objects,
        caches,
        projections,
        trims,
    });
}
