//! GCode.cpp:5754–5775: spiral orientation excludes holes, projected splitting does not.

use crate::runner::native_stream_case;
use serde_json::json;
use std::fs;

const DIR: &str = "spiral-hole-projected-split";
const UPSTREAM: &[&str] = &[
    "GCode.cpp:5754-5775",
    "ExtrusionEntity.cpp:230-320;PerimeterGenerator.cpp:124,528",
];
const CASES: [(&str, &str, &[u8]); 3] = [
    (
        "mk4-spiral-hole",
        "7737bc4991c554b645ceed6fe819462cfb1ece3ad2b6cfdf9095dacd78dbde27",
        include_bytes!("../../../../tests/parity/spiral-hole-projected-split/spiral.3mf"),
    ),
    (
        "mk4-stock-hole",
        "4c4ceaf15b083f1882c29ab0dc867ae62099abd3d3c13c30c8b6f6c2416766fa",
        include_bytes!("../../../../tests/parity/spiral-hole-projected-split/control.3mf"),
    ),
    (
        "cw-rectangle",
        "94a5b7e51d536faae94e4900b17f4eadf92fb0dea4c7c3d3f2c53c7f4231a48a",
        include_bytes!("../../../../tests/parity/spiral-placement-gate/cw-rectangle.3mf"),
    ),
];

#[test]
fn spiral_hole_projected_split() {
    let (label, sha, project) = CASES[0];
    native_stream_case::check((DIR, UPSTREAM), label, project, sha, (false, |_, _| {}));
}

#[test]
fn spiral_hole_stock_control() {
    let (label, sha, project) = CASES[1];
    native_stream_case::check((DIR, UPSTREAM), label, project, sha, (false, |_, _| {}));
}

#[test]
fn spiral_hole_native_preflight() {
    let namespace = format!("{DIR}-preflight");
    let mut qualified = Vec::new();
    for (label, sha, project) in CASES {
        let (reference, replay) =
            native_stream_case::qualify((&namespace, UPSTREAM), label, project, sha, false);
        qualified.push(json!({"case":label,"input_sha256":sha,
            "reference":reference.root,"replay":replay.root}));
    }
    fs::write(
        crate::artifacts::root_from_env()
            .unwrap()
            .join(format!("{namespace}.json")),
        serde_json::to_vec_pretty(&qualified).unwrap(),
    )
    .unwrap();
}
