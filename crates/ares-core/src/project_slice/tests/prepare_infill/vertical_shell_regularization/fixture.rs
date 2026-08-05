use crate::project_slice::prepare_infill::{
    vertical_shell_regularization, vertical_shell_trimming,
};

pub(in crate::project_slice::tests::prepare_infill) fn prepare(
    bytes: impl AsRef<[u8]>,
) -> vertical_shell_regularization::PreparedPostVerticalShellRegularization {
    vertical_shell_regularization::prepare(prepare_o21(bytes)).unwrap()
}

pub(super) fn prepare_o21(
    bytes: impl AsRef<[u8]>,
) -> vertical_shell_trimming::PreparedPostVerticalShellTrim {
    vertical_shell_trimming::prepare(super::super::vertical_shell_projection::prepare_fixture(
        bytes,
    ))
    .unwrap()
}
