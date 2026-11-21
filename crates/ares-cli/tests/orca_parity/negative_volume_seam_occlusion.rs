//! SeamPlacer global occlusion includes negative surfaces and all-hit visibility.

#[path = "negative_volume_seam_occlusion/timelapse.rs"]
mod timelapse;

const NEGATIVE_TARGET: &[u8] =
    include_bytes!("../../../../tests/parity/negative-volume-seam-occlusion/negative-target.3mf");

fn check(label: &str, project: &[u8], sha: &str) {
    crate::runner::native_stream_case::check(
        (
            "negative-volume-seam-occlusion",
            &[
                "GCode/SeamPlacer.cpp:128-233,628-704",
                "TriangleSetSampling.cpp:9-70",
                "AABBTreeIndirect.hpp:755-787",
            ],
        ),
        label,
        project,
        sha,
        (false, |_, _| {}),
    );
}

#[test]
fn negative_volume_occlusion_matches_native() {
    check(
        "negative-target",
        NEGATIVE_TARGET,
        "99c3499ffd505606a35441a3afe09c1289c4cb8a6132c7dd3050f3472e38f6db",
    );
}

#[test]
fn normal_part_occlusion_control_matches_native() {
    check(
        "normal-control",
        include_bytes!(
            "../../../../tests/parity/negative-volume-seam-occlusion/normal-control.3mf"
        ),
        "bd89b5a9a30499552c21977cd1f691804bcf63aaef301295b48bad35713bc092",
    );
}
