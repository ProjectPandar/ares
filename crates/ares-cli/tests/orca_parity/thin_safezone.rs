//! Upstream safe-zone miter default through complete Native/API/CLI streams.

#[test]
fn thin_auto_gap() {
    check(
        "thin-auto-gap-beam1",
        include_bytes!("../../../../tests/parity/interlocking-thin-gap/thin-auto-gap-beam1.3mf"),
        "a9cf6ad1fbef7df90556814c70791f8998a9c230c50a040cbde35381bee35e80",
    );
}

#[test]
fn thin_percent_gap() {
    check(
        "thin-percent-gap-beam1",
        include_bytes!("../../../../tests/parity/interlocking-thin-gap/thin-percent-gap-beam1.3mf"),
        "56144334ff023960cc36dfa5b59d936c9b88d3dd772917dd09808d88d1f925a4",
    );
}

#[test]
fn odd_kernel_control() {
    check(
        "odd-kernel-beam1",
        include_bytes!("../../../../tests/parity/endpoint-contact/odd-kernel-beam1.3mf"),
        "180d44d516f560510bb19a6584e6f41cd092cb16788a6a4091d7a32c69d841b7",
    );
}

#[test]
fn orthogonal_control() {
    check(
        "orthogonal-grid-beam1",
        include_bytes!("../../../../tests/parity/inward-displacement/orthogonal-grid-beam1.3mf"),
        "afed0f4f4417916d2963363754bc45ef1300f6bbc2d6dfbcc9d52cf751a0c312",
    );
}

#[test]
fn three_region_control() {
    check(
        "three-region-beam1",
        include_bytes!("../../../../tests/parity/interlocking-thin-gap/three-region-beam1.3mf"),
        "71a5035e7cdade9b98ecd96379fdee03ced8151dc54958e959e5b444f66e6701",
    );
}

fn check(label: &str, project: &[u8], sha: &str) {
    crate::runner::native_stream_case::check(
        (
            "thin-safezone",
            &[
                "GCode/AvoidCrossingPerimeters.cpp:init_layer:1324-1328",
                "ClipperUtils.hpp:DefaultMiterLimit:27/offset_ex:346",
            ],
        ),
        label,
        project,
        sha,
        (true, |_, _| {}),
    );
}
