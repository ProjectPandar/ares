//! Real processed slice -> conversion -> public WASM adapter, stable native authority.
//! Failure inventory: stale marker identity, extra phantom, G-code id drift,
//! wrong duration bits, exclusive/visible-only prefixes, drained timing storage.
use ares_core::{
    ConfigOverrides, GenerationMetadata, SlicerContext, slice_project_plate_processed,
};
use ares_wasm::SliceOutput;

#[path = "../../../tests/support/marker_time.rs"]
pub mod marker_time_native;

#[test]
fn required_missing_tagged_producer_cannot_pass_from_frozen_authority() {
    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "tagged_normal_time_matches_every_stable_native_vertex",
            "--nocapture",
        ])
        .env("ARES_REQUIRE_ORCA", "1")
        .env_remove("ARES_MARKER_TIME_STABLE_BIN")
        .output()
        .unwrap();
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .lines()
            .any(|line| line == "running 1 test"),
        "child selector did not run exactly one test: {output:?}"
    );
    assert!(
        !output.status.success(),
        "tagged used frozen authority without ARES_MARKER_TIME_STABLE_BIN"
    );
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("ARES_REQUIRE_ORCA=1: missing ARES_MARKER_TIME_STABLE_BIN")
    );
}

#[tokio::test]
async fn tagged_normal_time_matches_every_stable_native_vertex() {
    let Some(root) = marker_time_native::tagged_directory() else {
        return;
    };
    let overrides = ConfigOverrides::from_json(include_bytes!(
        "../../../tests/parity/marker-time/tagged-overrides.json"
    ))
    .unwrap();
    let processed = slice_project_plate_processed(
        include_bytes!("../../../tests/parity/orca_cli_ender3.3mf").to_vec(),
        GenerationMetadata::default(),
        Some(1),
        &overrides,
        SlicerContext::new(0),
    )
    .await
    .unwrap();
    let mut output = SliceOutput::from_processed(processed);
    let gcode = String::from_utf8(output.take_gcode()).unwrap();
    let vertices = output.take_vertices();
    let ids = vertices.ids_range(0, vertices.count() as u32).unwrap();
    let values = vertices.values_range(0, vertices.count() as u32).unwrap();
    vertices.tags_range(0, vertices.count() as u32).unwrap();
    marker_time_native::tagged_pair(&root, &gcode);
    let records_tsv = std::fs::read_to_string(root.join("stable.tsv")).unwrap();
    let records: Vec<_> = records_tsv.lines().collect();
    // No frozen Native row-count premise: the producer's stability is
    // asserted in tagged_pair (double run, byte-identical artifacts) and
    // the record count is compared as part of the whole-record comparison
    // below (vertex count == record count), never waived on its own.
    assert_eq!(vertices.count(), records.len());
    for (id, line) in records.iter().enumerate().rev() {
        let expected: Vec<u32> = line.split('\t').map(|v| v.parse().unwrap()).collect();
        assert_eq!(id as u32, expected[0], "stable id {id}");
        assert_eq!(ids[id * 2], expected[1], "gcode id at stable vertex {id}");
        for mode in 0..2 {
            assert_eq!(
                values[id * 16 + 10 + mode].to_bits(),
                expected[2 + mode],
                "duration mode {mode}, stable vertex {id}"
            );
        }
        let time = vertices.marker_time(id as u32).unwrap();
        assert_eq!(
            time[0].to_bits(),
            expected[4],
            "elapsed at stable vertex {id}"
        );
        assert_eq!(
            time[1].to_bits(),
            expected[2],
            "duration at stable vertex {id}"
        );
    }
    std::fs::write(root.join("rust.txt"), "stable ids/gcode ids; both duration modes; Normal elapsed bits; buffers transferred; exact match\n").unwrap();
}
