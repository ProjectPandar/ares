use crate::project_slice::prepare_infill::{horizontal_shell_promotion, vertical_shell_assignment};

pub(super) fn prepare_o24(
    bytes: impl AsRef<[u8]>,
) -> vertical_shell_assignment::PreparedPostVerticalShellAssignment {
    super::super::vertical_shell_assignment::fixture::prepare(bytes)
}

pub(in crate::project_slice::tests::prepare_infill) fn prepare(
    bytes: impl AsRef<[u8]>,
) -> horizontal_shell_promotion::PreparedPostHorizontalShellPromotion {
    horizontal_shell_promotion::prepare(prepare_o24(bytes)).unwrap()
}
