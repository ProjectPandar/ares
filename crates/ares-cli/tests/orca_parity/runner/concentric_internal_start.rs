//! FillConcentric.cpp:105-145 standard-variant loop start before stitch closure parity.
use crate::runner::native_stream_case::check;

const SOURCE: (&str, &[&str]) = (
    "concentric-internal-start",
    &[
        "Fill/FillConcentric.cpp:105-145",
        "Arachne/WallToolPaths.cpp:573-670",
        "Arachne/utils/ExtrusionLine.hpp:205-222",
    ],
);

#[test]
fn concentric_internal_start_monotonic_control_matches_native_complete_output() {
    check(
        SOURCE,
        "control",
        include_bytes!("../../../../../tests/parity/concentric-internal-start/control.3mf"),
        "b8bc9d8ab8db4eeaa21da61e759c64f230ddf4da800997424faf10fc53cfd298",
        (false, |_, _| {}),
    );
}

#[test]
fn concentric_internal_start_concentric_target_matches_native_complete_output() {
    check(
        SOURCE,
        "target",
        include_bytes!("../../../../../tests/parity/concentric-internal-start/target.3mf"),
        "2407dd48fea1529b51947958c6ea62d577d61f6b8173a989c1b4113983b6c6f6",
        (false, |_, _| {}),
    );
}
