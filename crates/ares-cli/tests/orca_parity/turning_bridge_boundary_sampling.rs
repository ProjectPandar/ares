use crate::runner::{native_api_cli::NativeCase, native_stream_case};

const UPSTREAM: (&str, &[&str]) = (
    "turning-bridge-boundary-sampling",
    &["PrintObject.cpp:3552-3749", "PrintObject.cpp:3959-4096"],
);

fn has_bridge_and_solid_support(reference: &NativeCase, _: &NativeCase) {
    for feature in ["Internal Bridge", "Internal solid infill", "Sparse infill"] {
        assert!(
            reference
                .native
                .lines()
                .filter_map(|line| {
                    line.strip_prefix(";TYPE:")
                        .or_else(|| line.strip_prefix("; FEATURE: "))
                })
                .any(|label| label == feature),
            "the input must print {feature}"
        );
    }
}

#[test]
fn hilbert_bridge_boundary_and_solid_support_match_native() {
    native_stream_case::check(
        UPSTREAM,
        "mk4-hilbert",
        include_bytes!("../../../../tests/parity/turning-bridge-boundary-sampling/mk4-hilbert.3mf"),
        "cd8721434d89f490adea13c5b9dcf14733a8f6ae2f2f5e33a475b54c82485961",
        (false, has_bridge_and_solid_support),
    );
}

#[test]
fn octagram_bridge_boundary_and_solid_support_match_native() {
    native_stream_case::check(
        UPSTREAM,
        "mk4-octagram",
        include_bytes!(
            "../../../../tests/parity/turning-bridge-boundary-sampling/mk4-octagram.3mf"
        ),
        "92948f63370945511a0c603bc994acab20292ea98b2e5358bd8b9d9ad000f199",
        (false, has_bridge_and_solid_support),
    );
}

#[test]
fn stock_bridge_boundary_and_solid_support_match_native() {
    native_stream_case::check(
        UPSTREAM,
        "mk4-stock",
        include_bytes!("../../../../tests/parity/turning-bridge-boundary-sampling/mk4-stock.3mf"),
        "95dca79f8de403d14ccb1ce9230776cd5f33c8dfa2a859a4d5091ca222ad98fe",
        (false, has_bridge_and_solid_support),
    );
}
