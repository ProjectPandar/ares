use crate::project_slice::prepare_infill::{vertical_shell_projection, vertical_shell_trimming};

pub(super) fn prepare(
    bytes: impl AsRef<[u8]>,
) -> vertical_shell_trimming::PreparedPostVerticalShellTrim {
    vertical_shell_trimming::prepare(prepare_o20(bytes)).unwrap()
}

pub(super) fn prepare_o20(
    bytes: impl AsRef<[u8]>,
) -> vertical_shell_projection::PreparedPostVerticalShellProjection {
    super::super::vertical_shell_projection::prepare_fixture(bytes)
}
