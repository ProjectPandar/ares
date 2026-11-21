use crate::runner::native_stream_case;

#[test]
fn overhang_wall_segment_retracts_without_wiping_at_zero_travel_minimum() {
    check(
        "zero-minimum-travel",
        include_bytes!(
            "../../../../tests/parity/overhang-wall-retract-wipe/zero-minimum-travel.3mf"
        ),
        "d2d2351bd7daf0cc4053ea4abfccde67c1ffd1ea6ef9af702f7c71649e666718",
    );
}

#[test]
fn overhang_wall_segment_retracts_without_wiping_when_avoiding_crossings() {
    check(
        "avoid-crossing-walls",
        include_bytes!(
            "../../../../tests/parity/overhang-wall-retract-wipe/avoid-crossing-walls.3mf"
        ),
        "44a0fda6b54554393abe41d6c70b0a7ab01766446618157cbb4688cd177c1e19",
    );
}

fn check(label: &str, project: &[u8], sha: &str) {
    native_stream_case::check(
        (
            "overhang-wall-retract-wipe",
            &[
                "GCode.cpp:8766-8768::extrude_loop",
                "GCode.cpp:10594-10623::travel_to",
                "GCode.cpp:10838-10865::retract",
            ],
        ),
        label,
        project,
        sha,
        (false, |_, _| {}),
    );
}
