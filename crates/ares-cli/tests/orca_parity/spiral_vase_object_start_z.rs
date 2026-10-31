//! `SpiralVase.cpp:131-141` + its producer: every Z-only G1 of a
//! spiral-enabled layer survives the vase filter rewritten to the ramp
//! base z (3-decimal `GCodeLine::set` splice, `GCodeReader.cpp:323-357`).
//! The producer is the generic writer mechanism: `change_layer` arms
//! `m_need_change_layer_lift_z` (`GCode.cpp:5708`) and `_extrude`'s guard
//! fires on the flag alone — even when the layer's first point equals the
//! current position — flushing the `M624` labels via the zero-length
//! `travel_to` and then emitting a forced
//! `travel_to_z(m_nominal_z, comment, /*force=*/ true)` that bypasses
//! `will_move_z` (`GCode.cpp:6367-6385`, `GCodeWriter.cpp:811-929`), so
//! the Z-only line appears at object starts with no XY travel. RED on this
//! tip: Ares drops those lines (the filter's per-layer dedup) — witness
//! fails the complete comparator at byte 81880 (47 Native-only bare
//! `G1 Z<z>` lines, `0.600 -> 9.800`); the `z_hop = 0` probe pins the
//! class as hop-independent (byte 81702).

use crate::runner::native_stream_case;
use serde_json::json;
use std::fs;

const DIR: &str = "spiral-vase-object-start-z";
const UPSTREAM: &[&str] = &[
    "SpiralVase.cpp:86-141",
    "GCodeReader.cpp:323-357",
    "GCode.cpp:5708,6367-6385,7350-7500,4599-4615,3698-3705,3748-3751",
    "GCodeWriter.cpp:811-848,916-929",
    "CoolingBuffer.cpp:823,901-960",
];
// The witness is the admission slice's committed single-object fixture
// (same bytes, sha `a8d0ed5f…`).
const CASES: [(&str, &str, &[u8]); 2] = [
    (
        "a1-single",
        "a8d0ed5f032f22a3b8968c9c648f621b913323abebaa417077f359b112b0ae38",
        include_bytes!("../../../../tests/parity/spiral-vase-admission/single-object-by-layer.3mf"),
    ),
    (
        "a1-single-zhop0",
        "27501792b3793d29974f773defb264d1de8342c87ebf8d7cc2189ef4ee8cd935",
        include_bytes!("../../../../tests/parity/spiral-vase-object-start-z/a1-single-zhop0.3mf"),
    ),
];

#[test]
fn spiral_object_start_z_witness() {
    let (label, sha, project) = CASES[0];
    native_stream_case::check((DIR, UPSTREAM), label, project, sha, (false, |_, _| {}));
}

#[test]
fn spiral_object_start_z_zhop0() {
    let (label, sha, project) = CASES[1];
    native_stream_case::check((DIR, UPSTREAM), label, project, sha, (false, |_, _| {}));
}

#[test]
fn spiral_object_start_z_native_preflight() {
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
