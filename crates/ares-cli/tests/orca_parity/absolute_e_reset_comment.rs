//! GCodeWriter.cpp:82–106,480–503: preamble reset_e(true) comment serialization.

use crate::runner::native_stream_case;
use serde_json::json;
use std::fs;

const DIR: &str = "absolute-e-reset-comment";
const UPSTREAM: &[&str] = &[
    "GCodeWriter.cpp:82-106,480-503;GCode.cpp:2038",
    "Tab.cpp:2753,4546,5470-5471",
];
const RESET: &str = "G92 E0 ; reset extrusion distance";
const CASES: [(&str, &str, &[u8], bool); 3] = [
    (
        "absolute-verbose",
        "9911f8075c51e6e9481132c2c4c5457ec494addf984740811a1c8896c9c3219f",
        include_bytes!("../../../../tests/parity/absolute-e-reset-comment/absolute-verbose.3mf"),
        true,
    ),
    (
        "absolute-quiet",
        "908b9d9ebb69b852bcba5d3c52d364b245350fdab31d067c0cdf69082cefbfde",
        include_bytes!("../../../../tests/parity/absolute-e-reset-comment/absolute-quiet.3mf"),
        false,
    ),
    (
        "relative-verbose",
        "ef1951bd9f85d84ebfaf569b5a0ac4cf729e22e3c0c0d788ea5ad71b54a6e632",
        include_bytes!("../../../../tests/parity/absolute-e-reset-comment/relative-verbose.3mf"),
        false,
    ),
];

#[test]
fn absolute_e_reset_verbose() {
    let (label, sha, project, expected) = CASES[0];
    native_stream_case::check(
        (DIR, UPSTREAM),
        label,
        project,
        sha,
        (false, |reference, replay| {
            for native in [reference, replay] {
                assert_eq!(native.native.contains(RESET), expected);
            }
        }),
    );
}

#[test]
fn absolute_e_reset_quiet_control() {
    let (label, sha, project, expected) = CASES[1];
    native_stream_case::check(
        (DIR, UPSTREAM),
        label,
        project,
        sha,
        (false, |reference, replay| {
            for native in [reference, replay] {
                assert_eq!(native.native.contains(RESET), expected);
            }
        }),
    );
}

#[test]
fn absolute_e_reset_relative_control() {
    let (label, sha, project, expected) = CASES[2];
    native_stream_case::check(
        (DIR, UPSTREAM),
        label,
        project,
        sha,
        (false, |reference, replay| {
            for native in [reference, replay] {
                assert_eq!(native.native.contains(RESET), expected);
            }
        }),
    );
}

#[test]
fn absolute_e_reset_native_preflight() {
    let namespace = format!("{DIR}-preflight");
    let mut qualified = Vec::new();
    for (label, sha, project, expected) in CASES {
        let (reference, replay) =
            native_stream_case::qualify((&namespace, UPSTREAM), label, project, sha, false);
        for native in [&reference, &replay] {
            assert_eq!(native.native.contains(RESET), expected);
        }
        qualified.push(json!({"case":label,"input_sha256":sha,
            "reference":reference.root,"replay":replay.root,
            "expected_reset_comment":expected}));
    }
    fs::write(
        crate::artifacts::root_from_env()
            .unwrap()
            .join(format!("{namespace}.json")),
        serde_json::to_vec_pretty(&qualified).unwrap(),
    )
    .unwrap();
}
