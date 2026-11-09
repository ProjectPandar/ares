//! TriangleMeshSlicer.cpp:603-635,1616-1666,1997-2038: mesh-local raw lines.

use std::fs;

use crate::{
    OrcaRunner,
    runner::{
        native_api_cli::{NativeCase, sha},
        native_stream_case,
    },
};

#[path = "raw_intersection_capacity/mesh.rs"]
mod mesh;

const CONTROL: &[u8] = include_bytes!(
    "../../../../tests/parity/machine-acceleration-cache-rounding/stock-control.3mf"
);
const UPSTREAM: &[&str] = &[
    "TriangleMeshSlicer.cpp:603-635,1616-1666,1997-2038,2142-2186",
    "PrintObjectSlice.cpp:slice_volume",
];

#[test]
fn runtime_dense_stock_mesh_complete_parity() {
    let (project, density) = mesh::subdivide(CONTROL);
    assert!(density["raw_intersections"].as_u64().unwrap() > 1_000_000);
    let (reference, _) = native_stream_case::qualify(
        ("raw-intersection-capacity", UPSTREAM),
        "stock-x-strips",
        &project,
        &sha(&project),
        false,
    );
    fs::write(
        reference.root.join("mesh-density.json"),
        serde_json::to_vec_pretty(&density).unwrap(),
    )
    .unwrap();
    let runner = OrcaRunner::from_env().unwrap().unwrap();
    let control = NativeCase::slice(&runner, CONTROL, "raw-intersection-stock-control-");
    let metadata = ares_core::GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0);
    control.compare(metadata);
    reference.compare(metadata);
}
