//! MK4 `make_overhang_printable=1` witness for the curled-perimeter
//! lifecycle slice (`docs/architecture/mk4-curl-lifecycle-spec.md`).
//!
//! Stock Prusa MK4 defaults with `make_overhang_printable=1`. Native runs
//! `PrintObject::estimate_curled_extrusions` (`PrintObject.cpp:897-913`)
//! over the ORIGINAL ordered perimeters with the nominal
//! `frExternalPerimeter` role flow BEFORE `simplify_extrusion_path`
//! (`Print.cpp:2303/2393` before `:2571`) and before seam placement
//! (`GCode.cpp:5752-5775`). At
//! `345efec64c7b95a05ce5cf96df0dde66a158835c`, Native enables the curl
//! estimator through `PrintObject.cpp:1031-1050`; perimeter speed selection
//! uses the live role speed (`GCode.cpp:9553-9626`). Check enabled inputs
//! and later-layer outer-wall slowdown/variation, not a feedrate/count golden.
//! Cooling can also change feedrates; this is not a curl-only causal witness.
//! The unchanged whole-stream comparisons cover every motion and progress byte.

use crate::runner::native_stream_case;

const UPSTREAM: &[&str] = &[
    "PrintObject.cpp:897-913;SupportSpotsGenerator.cpp:141-196,92",
    "Print.cpp:2303,2393,2571;GCode.cpp:5752-5775;ExtrusionProcessor.hpp:296-308",
];

fn assert_native_witness(native: &str) {
    let option = |name| {
        let prefix = format!("; {name} = ");
        native
            .lines()
            .find_map(|line| line.strip_prefix(&prefix))
            .unwrap()
    };
    for key in [
        "make_overhang_printable",
        "enable_overhang_speed",
        "slowdown_for_curled_perimeters",
        "use_relative_e_distances",
    ] {
        assert_eq!(option(key), "1", "Native enabled input {key}");
    }
    let nominal_feedrate = option("outer_wall_speed").parse::<f64>().unwrap() * 60.0;
    let mut seen_layer = false;
    let mut later_layer = false;
    let mut outer_wall = false;
    let mut feedrate = 0.0;
    let mut first_feedrate = None;
    let mut slowed = false;
    let mut varied = false;
    for line in native.lines() {
        if line == ";LAYER_CHANGE" {
            later_layer = seen_layer;
            seen_layer = true;
        }
        if let Some(role) = line.strip_prefix(";TYPE:") {
            outer_wall = role == "Outer wall";
        }
        let mut words = line.split(';').next().unwrap().split_ascii_whitespace();
        let Some(command) = words.next() else {
            continue;
        };
        if !matches!(command, "G0" | "G1" | "G2" | "G3") {
            continue;
        }
        if let Some(value) = words.clone().find_map(|word| word.strip_prefix('F')) {
            feedrate = value.parse::<f64>().unwrap();
        }
        let extrudes = words.clone().any(|word| {
            word.strip_prefix('E')
                .is_some_and(|value| value.parse::<f64>().unwrap() > 0.0)
        });
        let xy = words
            .clone()
            .any(|word| word.starts_with('X') || word.starts_with('Y'));
        if later_layer && outer_wall && extrudes && xy {
            slowed |= feedrate < nominal_feedrate;
            varied |= first_feedrate.is_some_and(|first| first != feedrate);
            first_feedrate.get_or_insert(feedrate);
        }
    }
    assert!(
        slowed && varied,
        "Native later-layer outer-wall speed activity"
    );
}

#[test]
fn mk4_curl_lifecycle_strict() {
    native_stream_case::check(
        ("mk4-curl-lifecycle", UPSTREAM),
        "make-overhang-printable",
        include_bytes!("../../../../tests/parity/mk4-curl-lifecycle/witness.3mf"),
        "43ee2c6d9574c291cad8f3530b7dc233c3c38c7243bc8f0c365296848ada5549",
        (false, |reference, replay| {
            for native in [reference, replay] {
                assert_native_witness(&native.native);
            }
        }),
    );
}
