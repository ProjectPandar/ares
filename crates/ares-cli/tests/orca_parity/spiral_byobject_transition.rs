//! `SpiralVase.cpp:120-121,154-165,205-216` + `GCode.cpp:3805-3806`: under
//! by-object printing every object's own final layer carries the
//! `transition_out` flat taper pass — the spiral filter clones each ramp
//! line into a flat closing layer with `E × finishing factor` (A1
//! `spiral_finishing_flow_ratio = 0` → closing `E0.00000`) and appends it
//! after the ramp block (`new_gcode + transition_gcode`). Ares gates
//! `final_layer` on the plate-final schedule entry
//! (`layers/object_entry.rs:154`), so the FIRST object of a by-object
//! print loses the pass: on the witness Native prints 83 layer blocks
//! (`; total layer number: 83`, `overhang.obj` top layer ramp + flat
//! taper) while Ares prints 82. RED fails the complete comparator at
//! line 5. The witness reuses the admission slice's committed fixture.

use crate::runner::native_stream_case;
use serde_json::json;
use std::fs;

const DIR: &str = "spiral-byobject-transition";
const UPSTREAM: &[&str] = &[
    "SpiralVase.cpp:120-121,154-165,205-216",
    "GCode.cpp:3772+,3698-3705,3805-3806",
    "Print.cpp:1334-1341",
];
const WITNESS: (&str, &str, &[u8]) = (
    "byobject-apart",
    "2dbf797da35adf51f085c397cfa959364fd7411a73ef52ec687d04dd480e7342",
    include_bytes!("../../../../tests/parity/spiral-vase-admission/two-object-by-object-apart.3mf"),
);

#[test]
fn orca_parity_spiral_byobject_transition_witness() {
    let (label, sha, project) = WITNESS;
    native_stream_case::check((DIR, UPSTREAM), label, project, sha, (false, |_, _| {}));
}

#[test]
fn orca_parity_spiral_byobject_transition_native_preflight() {
    let (label, sha, project) = WITNESS;
    let namespace = format!("{DIR}-preflight");
    let (reference, replay) =
        native_stream_case::qualify((&namespace, UPSTREAM), label, project, sha, false);
    fs::write(
        crate::artifacts::root_from_env()
            .unwrap()
            .join(format!("{namespace}.json")),
        serde_json::to_vec_pretty(&json!({"case":label,"input_sha256":sha,
            "reference":reference.root,"replay":replay.root}))
        .unwrap(),
    )
    .unwrap();
}
