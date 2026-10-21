//! Instance names retain the source replace-runs then single-edge erasure.

#[test]
fn leading_special_instance_name_matches_native() {
    crate::runner::native_stream_case::check(
        (
            "instance-name-sanitize",
            &["GCode.cpp:4364-4382::sanitize_instance_name/get_instance_name"],
        ),
        "leading-special-target",
        include_bytes!(
            "../../../../tests/parity/instance-name-sanitize/leading-special-target.3mf"
        ),
        "72ffb0bf2f159c54f21577abe5ccb650521f191f3bcea3fd5e514879ab27fc67",
        false,
    );
}

#[test]
fn plain_instance_name_sole_key_control() {
    crate::runner::native_stream_case::check(
        (
            "instance-name-sanitize",
            &["GCode.cpp:4364-4382::sanitize_instance_name/get_instance_name"],
        ),
        "plain-name-control",
        include_bytes!("../../../../tests/parity/instance-name-sanitize/plain-name-control.3mf"),
        "b28411348c6bd4dc30c5ec412403f389697fd8362026a64472b4926ec4a635b9",
        false,
    );
}
