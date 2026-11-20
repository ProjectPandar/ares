#![cfg(target_os = "linux")]

use super::direct_stl_slice::compare_slice;

#[test]
fn cli_direct_stl_arranges_separate_inputs_by_default() {
    compare_slice("1", 2, None, false);
}

#[test]
fn cli_direct_stl_arranges_separate_inputs_when_forced() {
    compare_slice("0", 2, Some("1"), false);
}
