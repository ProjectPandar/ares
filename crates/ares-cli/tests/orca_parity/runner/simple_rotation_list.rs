//! Fill.cpp:206-210 / Config.hpp:868-890: non-meta comma-item interpretation.

use super::native_stream_case::check;

const SOURCE: (&str, &[&str]) = (
    "simple-rotation-list",
    &[
        "libslic3r/Fill/Fill.cpp:63-68,206-210",
        "libslic3r/Config.hpp:868-900",
    ],
);

#[test]
fn space_separated_simple_template_matches_native_complete_output() {
    check(
        SOURCE,
        "space-list",
        include_bytes!("../../../../../tests/parity/simple-rotation-list/space-list.3mf"),
        "703ea50de971fcfd681ff41e40aadc43e7c3cfab8db13a884da0c8e0a0e268af",
        (false, |_, _| {}),
    );
}

#[test]
fn tab_separated_simple_template_matches_native_complete_output() {
    check(
        SOURCE,
        "tab-list",
        include_bytes!("../../../../../tests/parity/simple-rotation-list/tab-list.3mf"),
        "ceeb9090d40c3bae21884e06e74994e69d0a694be11da05a7606ebc60b8c0396",
        (false, |_, _| {}),
    );
}

#[test]
fn comma_list_control_preserves_complete_output() {
    check(
        SOURCE,
        "comma-list",
        include_bytes!("../../../../../tests/parity/simple-rotation-list/comma-list.3mf"),
        "aa4bf45f18ef6c40f9e80953e7e7857f952a62e1c30fa78e8814c14c802b0799",
        (false, |_, _| {}),
    );
}

#[test]
fn single_angle_control_preserves_complete_output() {
    check(
        SOURCE,
        "single-angle",
        include_bytes!("../../../../../tests/parity/simple-rotation-list/single-angle.3mf"),
        "5ba6ae35c3f589214c84f51d32123861dc9666dd8f46cc8acf76b7406f0937d1",
        (false, |_, _| {}),
    );
}

#[test]
fn meta_space_control_preserves_complete_output() {
    check(
        SOURCE,
        "meta-space-control",
        include_bytes!("../../../../../tests/parity/simple-rotation-list/meta-space-control.3mf"),
        "1e23f416cc2deb8824981740a4a59cd3cb3dd50700b954b235cffbdb8d0d2220",
        (false, |_, _| {}),
    );
}
