//! `SeamPlacer.cpp:1581`: the concave-corner seam gate compares
//! `local_ccw_angle < -EPSILON` with the shared `EPSILON = 1e-4`
//! double literal (`libslic3r.h:52`), not the f32 machine epsilon.

use crate::runner::native_stream_case::check;

#[test]
fn seam_position_random_concave_epsilon_matches_complete_output() {
    // Seeded-only oracle: two official replays of the single-object KSR
    // project disagree on the uninitialized `; printing object` id, so
    // `native_stream_case::qualify` would panic with Native
    // nondeterminism in official mode (same qualification as
    // `initialized_object_id.rs`).
    if std::env::var("ARES_PARITY_SEEDED_REFERENCE").as_deref() != Ok("1") {
        crate::native_environment::skipped("seeded-only reference");
        return;
    }
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
