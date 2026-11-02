//! OrcaSlicer beta `GCodeWriter.cpp::_spiral_travel_to_z:1262-1274`:
//! retain the arc-enabled branch in complete public output parity.

use crate::runner::native_stream_case;

#[test]
fn spiral_lift_arc_enabled_sequential_matches_native_complete_output() {
    check(
        "sequential",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/parity/spiral-lift-arc-enabled/sequential.3mf"
        )),
        "baa6c8d352d5e62c400cc450d50de643ce868fcc619a71352748f89aabe7aab1",
    );
}

#[test]
fn spiral_lift_arc_enabled_stock_matches_native_complete_output() {
    check(
        "stock",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/parity/spiral-lift-arc-enabled/stock.3mf"
        )),
        "b3f72543f8794281e1d99ef784a993dbb0523bfbe96468403cec4b21da34a5a2",
    );
}

#[test]
fn spiral_lift_arc_enabled_fuzzy_matches_native_complete_output() {
    check(
        "fuzzy",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/parity/spiral-lift-arc-enabled/fuzzy.3mf"
        )),
        "5d4a5f95fd4c12244e87b17e9e64052f56d66fd83df405438fb4c4e34c8c524c",
    );
}

fn check(label: &str, project: &[u8], sha: &str) {
    native_stream_case::check(
        (
            "spiral-lift-arc-enabled",
            &[
                "GCodeWriter.cpp::_spiral_travel_to_z:1262-1274",
                "GCodeWriter.hpp::GCodeG2G3Formatter:419-430",
            ],
        ),
        label,
        project,
        sha,
        (false, |reference, _| {
            assert!(
                reference
                    .native
                    .lines()
                    .any(|line| line == "G17" || line.starts_with("G17 ;"))
            );
            assert!(
                reference
                    .native
                    .lines()
                    .any(|line| line.starts_with("G3 Z"))
            );
        }),
    );
}
