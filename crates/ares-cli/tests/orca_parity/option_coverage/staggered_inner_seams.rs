//! `SeamPlacer.cpp:1594`: the convex-corner stagger depth recompute
//! `depth = depth * beta_angle / 1.4142` evaluates the multiply in f32,
//! promotes the product to double for the division by the `1.4142`
//! double literal, then narrows back to float.

use crate::runner::native_stream_case::check;

#[test]
fn staggered_inner_seams_convex_depth_matches_complete_output() {
    // Seeded-only oracle: two official replays of the single-object KSR
    // project disagree on the uninitialized `; printing object` id, so
    // `native_stream_case::qualify` would panic with Native
    // nondeterminism in official mode (same qualification as
    // `initialized_object_id.rs`).
    if !crate::runner::seeded_reference_enabled(None) {
        return;
    }
    check(
        ("staggered-inner-seams", &["SeamPlacer.cpp:1594"]),
        "x1c-staggered-inner-seams",
        include_bytes!(
            "../../../../../tests/parity/staggered-inner-seams/x1c-staggered-inner-seams.3mf"
        ),
        "54fa91c0ee36adfc6d7fe9e5b2f9ce503873c216473ee1fad7986914afda6e85",
        (false, |_, _| {}),
    );
}
