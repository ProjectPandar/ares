//! `GCode.cpp:7634-7640` internal-region suppression, evaluated on the routed travel polyline
//! (`GCode.cpp:7421-7427`). See
//! `docs/architecture/2026-10-21-retract-internal-regions-routed-spec.md`.

#[test]
fn default_prisms_routed_travel_suppresses_retraction() {
    check(
        "default-prisms-routed",
        include_bytes!("../../../../tests/parity/arachne/default-prisms.3mf"),
        "ed368d762de6a45de4e10cde08f35b15854de7a132bf37f0aaa8440bc011ea24",
        false,
    );
}

/// Sole-key control: the same fixture with `reduce_crossing_wall` forced to `0`, so upstream never
/// routes and both sides decide on the straight segment. It must stay byte-strict, which proves the
/// target's divergence is gated on routing and not on the fixture in general.
#[test]
fn reduce_crossing_wall_off_control() {
    check(
        "default-prisms-no-crossing-wall",
        include_bytes!("../../../../tests/parity/arachne/default-prisms-no-crossing-wall.3mf"),
        "63a343bb07586edc20e5af27b30081ff75f8502526f86c89b3f3680d8ebc15af",
        false,
    );
}

fn check(label: &str, project: &[u8], sha: &str, two_tools: bool) {
    crate::runner::native_stream_case::check(
        (
            "retract-internal-regions-routed",
            &[
                "GCode.cpp:travel_to:7350/:7359/:7415-7427",
                "GCode.cpp:needs_retraction:7528/:7634-7640",
                "GCode/RetractWhenCrossingPerimeters.cpp:travel_inside_internal_regions:9",
            ],
        ),
        label,
        project,
        sha,
        (two_tools, |_, _| {}),
    );
}
