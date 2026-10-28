//! `SeamPlacer.cpp:1581`: the concave-corner seam gate compares
//! `local_ccw_angle < -EPSILON` with the shared `EPSILON = 1e-4`
//! double literal (`libslic3r.h:52`), not the f32 machine epsilon.

use crate::runner::native_stream_case::check;

#[test]
fn orca_parity_seam_position_random_concave_epsilon_matches_complete_output() {
    check(
        (
            "seam-position-random",
            &["libslic3r.h:52", "SeamPlacer.cpp:1581"],
        ),
        "x1c-seam-position-random",
        include_bytes!(
            "../../../../../tests/parity/seam-position-random/x1c-seam-position-random.3mf"
        ),
        "11c3ec4b666ec546f899debb200fcddb5e18e97221a6a7f46694459c4e499534",
        (false, |_, _| {}),
    );
}
