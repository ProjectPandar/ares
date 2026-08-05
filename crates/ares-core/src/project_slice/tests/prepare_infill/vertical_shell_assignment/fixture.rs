use crate::project_slice::prepare_infill::{vertical_shell_assignment, vertical_shell_filtering};

pub(super) fn prepare(
    bytes: impl AsRef<[u8]>,
) -> vertical_shell_assignment::PreparedPostVerticalShellAssignment {
    vertical_shell_assignment::prepare(prepare_o23(bytes)).unwrap()
}

pub(super) fn prepare_o23(
    bytes: impl AsRef<[u8]>,
) -> vertical_shell_filtering::PreparedPostVerticalShellFiltering {
    super::super::vertical_shell_filtering::fixture::prepare(bytes)
}
