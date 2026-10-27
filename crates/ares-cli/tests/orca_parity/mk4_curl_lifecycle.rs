//! MK4 `make_overhang_printable=1` witness for the curled-perimeter
//! lifecycle slice (`docs/architecture/mk4-curl-lifecycle-spec.md`).
//!
//! Stock Prusa MK4 defaults with `make_overhang_printable=1`. Native runs
//! `PrintObject::estimate_curled_extrusions` (`PrintObject.cpp:897-913`)
//! over the ORIGINAL ordered perimeters with the nominal
//! `frExternalPerimeter` role flow BEFORE `simplify_extrusion_path`
//! (`Print.cpp:2303/2393` before `:2571`) and before seam placement
//! (`GCode.cpp:5752-5775`). Strict whole-stream parity: the
//! `G1 F1650` curl slowdown set (7 occurrences, no `G1 F1654`) is the
//! native witness that the fixture exercises the curled-perimeter
//! slowdowns this slice owns.

use crate::runner::native_stream_case;

const UPSTREAM: &[&str] = &[
    "PrintObject.cpp:897-913;SupportSpotsGenerator.cpp:141-196,92",
    "Print.cpp:2303,2393,2571;GCode.cpp:5752-5775;ExtrusionProcessor.hpp:296-308",
];

#[test]
fn orca_parity_mk4_curl_lifecycle_strict() {
    native_stream_case::check(
        ("mk4-curl-lifecycle", UPSTREAM),
        "make-overhang-printable",
        include_bytes!("../../../../tests/parity/mk4-curl-lifecycle/witness.3mf"),
        "43ee2c6d9574c291cad8f3530b7dc233c3c38c7243bc8f0c365296848ada5549",
        (false, |reference, replay| {
            for native in [reference, replay] {
                assert_eq!(
                    native
                        .native
                        .lines()
                        .filter(|&line| line == "G1 F1650")
                        .count(),
                    7,
                    "Native curl slowdown witness G1 F1650 x7 at {:?}",
                    native.root
                );
                assert!(
                    !native.native.lines().any(|line| line == "G1 F1654"),
                    "Native must not emit the pre-fix Ares value G1 F1654 at {:?}",
                    native.root
                );
            }
        }),
    );
}
