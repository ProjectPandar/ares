//! GCode.cpp:9458–9464: Brim uses first-layer speed, without masking progress.

#[tokio::test]
async fn below_threshold_brim_preserves_complete_executable_block() {
    compare_executable(
        "below-threshold",
        include_bytes!(
            "../../../../../tests/parity/clipper-wrapper-geometry/round-offset-short-edge-below-threshold.3mf"
        ),
        "ce8aa1e13b8f867acad2743d33cdec3a667018613066e6701ec0622f00eeeb97",
    )
    .await;
}

#[tokio::test]
async fn at_threshold_brim_preserves_complete_executable_block() {
    compare_executable(
        "at-threshold",
        include_bytes!(
            "../../../../../tests/parity/clipper-wrapper-geometry/round-offset-short-edge-at-threshold.3mf"
        ),
        "99a0afc16380d62a2da6dd956a338ecc23a592b5586297274cb7e17f8907ec9a",
    )
    .await;
}

#[tokio::test]
async fn above_threshold_brim_preserves_complete_executable_block() {
    compare_executable(
        "above-threshold",
        include_bytes!(
            "../../../../../tests/parity/clipper-wrapper-geometry/round-offset-short-edge-above-threshold.3mf"
        ),
        "6b53a7623b9dfca2271c8d138aca5f2b667693d145f7da648f15836f3902fe24",
    )
    .await;
}

async fn compare_executable(label: &str, project: &[u8], sha: &str) {
    let settings = crate::runner::application::settings(project).unwrap();
    assert_ne!(
        settings["initial_layer_speed"],
        settings["initial_layer_infill_speed"]
    );
    let (reference, _) = crate::runner::native_stream_case::qualify(
        (
            "brim-first-layer-speed",
            &[
                "345efec64c7b95a05ce5cf96df0dde66a158835c",
                "GCode.cpp:9458-9464 _extrude first-layer/over-raft Brim speed",
                "GCode.cpp:9466-9489 distinct is_perimeter-only slowdown ramp",
                "ExtrusionEntity.hpp:62-67 is_perimeter",
            ],
        ),
        label,
        project,
        sha,
        false,
    );
    let native_blocks = executable_blocks(&reference.native);
    assert_eq!(native_blocks.len(), 1);
    let native = native_blocks[0];
    assert!(native.contains(";TYPE:Brim\n"));
    std::fs::write(reference.root.join("native.executable.gcode"), native).unwrap();
    let api = crate::runner::application::slice_project_plate_validated(
        project,
        ares_core::GenerationMetadata::deterministic(2026, 10, 31, 0, 0, 0),
        None,
        &ares_core::ConfigOverrides::default(),
        ares_core::SlicerContext::new(0),
    )
    .await
    .unwrap();
    std::fs::write(reference.root.join("ares.api.gcode"), &api).unwrap();
    let api_blocks = executable_blocks(std::str::from_utf8(&api).unwrap());
    assert_eq!(api_blocks.len(), 1);
    let actual = api_blocks[0];
    std::fs::write(reference.root.join("ares.executable.gcode"), actual).unwrap();
    std::fs::write(
        reference.root.join("executable-selection.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "native_ordinal": 0,
            "api_ordinal": 0,
            "native_executable_blocks": native_blocks.len(),
            "api_executable_blocks": api_blocks.len(),
            "all_records_included": true,
            "boundary_markers_included": true,
        }))
        .unwrap(),
    )
    .unwrap();
    assert_eq!(
        actual, native,
        "Complete executable block ordinal0 must match, including all M73, feeds, moves, E, wipe, fan and end G-code: {:?}",
        reference.root,
    );
}

fn executable_blocks(gcode: &str) -> Vec<&str> {
    const START: &str = "; EXECUTABLE_BLOCK_START\n";
    const END: &str = "; EXECUTABLE_BLOCK_END\n";
    gcode
        .match_indices(START)
        .map(|(offset, _)| {
            let tail = &gcode[offset..];
            let end = tail.find(END).unwrap() + END.len();
            &tail[..end]
        })
        .collect()
}
