//! OrcaSlicer beta `GCodeWriter.cpp::_spiral_travel_to_z:1200-1275`
//! and `Geometry/ArcWelder.hpp:48-72`: complete public output parity.

use crate::runner::native_stream_case;

const VELOCITY: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/parity/spiral-lift-resolution/velocity-limit.3mf"
));
const POSITIVE_SYNC: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/parity/spiral-lift-resolution/positive-sync.3mf"
));

#[test]
fn spiral_lift_velocity_limit_matches_native_complete_output() {
    check(
        "velocity-limit",
        VELOCITY,
        "d8f74deff1274800b04f021ba812c5fb84c1629e45981653218963e2cc3a1e37",
    );
}

#[test]
fn spiral_lift_positive_sync_matches_native_complete_output() {
    check(
        "positive-sync",
        POSITIVE_SYNC,
        "182857ce98b97df3d76968284532f516779822d2c2eb7601b0522438990b4af2",
    );
}

fn check(label: &str, project: &[u8], sha: &str) {
    native_stream_case::check(
        (
            "spiral-lift-resolution",
            &[
                "GCodeWriter.cpp::_spiral_travel_to_z:1200-1275",
                "Geometry/ArcWelder.hpp::arc_discretization_steps:48-72",
                "GCodeWriter.hpp::GCodeFormatter:320-403",
            ],
        ),
        label,
        project,
        sha,
        (false, |reference, _| {
            for marker in [
                "; enable_arc_fitting = disabled\n",
                "; z_hop_types = Auto Lift\n",
                "; resolution = 0.012\n",
            ] {
                assert!(reference.native.contains(marker));
            }
        }),
    );
}
