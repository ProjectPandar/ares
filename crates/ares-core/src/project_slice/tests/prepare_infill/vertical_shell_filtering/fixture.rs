use crate::project_slice::prepare_infill::{
    vertical_shell_filtering, vertical_shell_regularization,
};

pub(in crate::project_slice) fn prepare(
    bytes: impl AsRef<[u8]>,
) -> vertical_shell_filtering::PreparedPostVerticalShellFiltering {
    vertical_shell_filtering::prepare(prepare_o22(bytes)).unwrap()
}

pub(super) fn prepare_o22(
    bytes: impl AsRef<[u8]>,
) -> vertical_shell_regularization::PreparedPostVerticalShellRegularization {
    super::super::vertical_shell_regularization::fixture::prepare(bytes)
}
