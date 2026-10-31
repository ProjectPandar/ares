//! Brim.cpp:813–856 public full-stream RED; GREEN is not authorized.
use crate::runner::native_stream_case;

#[test]
fn p1s_bylayer_brim_contour_matches_native() {
    native_stream_case::check(
        (
            "p1s-bylayer-brim",
            &["Brim.cpp:813-856::tryExPolygonOffset/makeBrimInfillImpl"],
        ),
        "target",
        include_bytes!("../../../../tests/parity/p1s-bylayer-brim/project.3mf"),
        "dcae275709dc16016c60308ae42d8523385dbabc18e5d31448f0baceb6e2d36a",
        (false, |_, _| {}),
    );
}

#[test]
fn p1s_bylayer_brim_outeronly_control_matches_native() {
    native_stream_case::check(
        (
            "p1s-bylayer-brim",
            &["Brim.cpp:813-856::tryExPolygonOffset/makeBrimInfillImpl"],
        ),
        "control",
        include_bytes!(
            "../../../../tests/parity/validation/brim_support_and_painted_ears/outer-only.3mf"
        ),
        "a4a661702406d16509ed439e704661cc8cc0e8f0bcc39695ed86218a2f26c9bc",
        (false, |_, _| {}),
    );
}
