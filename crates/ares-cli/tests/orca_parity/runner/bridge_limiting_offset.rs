use super::super::native_stream_case;

#[test]
fn scaled_limiting_boundary_matches_native_complete_output() {
    native_stream_case::check(
        (
            "bridge-limiting-offset",
            &[
                "PrintObject.cpp:3898-3903 bridge_over_infill boundary offsets",
                "Flow.hpp:71 scaled_spacing",
            ],
        ),
        "absolute-equal",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/parity/seam-gap-fill-group-key/absolute-equal.3mf"
        )),
        "7620eafdbeb793c46cfcf7745e3e8c94d5bdbd76cd8f7e8aa07f5e2ccd6dde8b",
        (false, |_, _| {}),
    );
}
