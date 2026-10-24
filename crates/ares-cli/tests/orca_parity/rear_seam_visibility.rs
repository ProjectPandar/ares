//! Rear visibility stays at the upstream default before seam alignment.

#[test]
fn aligned_stock_control_matches_native() {
    check(
        "aligned-control",
        include_bytes!("../../../../tests/parity/rear-seam-visibility/aligned-control.3mf"),
        "dc37585d4483eae1f5eec7331d395c99132b3bd5959ec96d486e1c901c1f0d89",
    );
}

#[test]
fn rear_zero_visibility_matches_native() {
    check(
        "rear-target",
        include_bytes!("../../../../tests/parity/rear-seam-visibility/rear-target.3mf"),
        "747a5ed4ac32c8d9d070ffd126f8da559a5d0c72554db84de203300a6dba2a47",
    );
}

fn check(label: &str, project: &[u8], expected_sha: &str) {
    crate::runner::native_stream_case::check(
        (
            "rear-seam-visibility",
            &[
                "GCode/SeamPlacer.hpp:58-68 SeamCandidate visibility default zero",
                "GCode/SeamPlacer.cpp:1427-1457 init visibility mode gate omits rear",
                "GCode/SeamPlacer.cpp:777-799,1272,1364-1391 rear comparison/alignment",
            ],
        ),
        label,
        project,
        expected_sha,
        (false, |_, _| {}),
    );
}
