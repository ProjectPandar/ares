//! GCode.cpp:4882–4935: complete non-BBL post-config statistics tail through EOF.

#[tokio::test]
async fn nested_island_preserves_complete_post_config_tail() {
    compare_tail(
        "nested-hole-island",
        include_bytes!(
            "../../../../../tests/parity/clipper-wrapper-geometry/nested-hole-island.3mf"
        ),
        "f91fae699cd1410003fe4c0d59ce2602c0fa1039e709c11d195641e5488da74c",
    )
    .await;
}

#[tokio::test]
async fn compensated_holes_preserve_complete_post_config_tail() {
    compare_tail(
        "holes-touch-after-compensation",
        include_bytes!(
            "../../../../../tests/parity/clipper-wrapper-geometry/holes-touch-after-compensation.3mf"
        ),
        "a1326b0b6be5f1f90d5e5a4ea618e86a3b8892eb137b17308838310c97474c1a",
    )
    .await;
}

#[tokio::test]
async fn collinear_boundary_preserves_complete_post_config_tail() {
    compare_tail(
        "collinear-boundary-with-thin-spike",
        include_bytes!(
            "../../../../../tests/parity/clipper-wrapper-geometry/collinear-boundary-with-thin-spike.3mf"
        ),
        "e3c2b3e5d29ac1766eca1be0464c1e8a0e88fe964630cb218f185affc504b1d5",
    )
    .await;
}

#[tokio::test]
async fn overhang_widths_preserve_complete_post_config_tail() {
    compare_tail(
        "coincident-width-overhang-intersections",
        include_bytes!(
            "../../../../../tests/parity/clipper-wrapper-geometry/coincident-width-overhang-intersections.3mf"
        ),
        "7688c5eaac8bb610fae4e92863474c85262d9f83f64737ba17f5574c6b635866",
    )
    .await;
}

#[tokio::test]
async fn below_threshold_brim_preserves_complete_post_config_tail() {
    compare_tail(
        "round-offset-short-edge-below-threshold",
        include_bytes!(
            "../../../../../tests/parity/clipper-wrapper-geometry/round-offset-short-edge-below-threshold.3mf"
        ),
        "ce8aa1e13b8f867acad2743d33cdec3a667018613066e6701ec0622f00eeeb97",
    )
    .await;
}

#[tokio::test]
async fn at_threshold_brim_preserves_complete_post_config_tail() {
    compare_tail(
        "round-offset-short-edge-at-threshold",
        include_bytes!(
            "../../../../../tests/parity/clipper-wrapper-geometry/round-offset-short-edge-at-threshold.3mf"
        ),
        "99a0afc16380d62a2da6dd956a338ecc23a592b5586297274cb7e17f8907ec9a",
    )
    .await;
}

#[tokio::test]
async fn above_threshold_brim_preserves_complete_post_config_tail() {
    compare_tail(
        "round-offset-short-edge-above-threshold",
        include_bytes!(
            "../../../../../tests/parity/clipper-wrapper-geometry/round-offset-short-edge-above-threshold.3mf"
        ),
        "6b53a7623b9dfca2271c8d138aca5f2b667693d145f7da648f15836f3902fe24",
    )
    .await;
}

async fn compare_tail(label: &str, project: &[u8], sha: &str) {
    let (reference, _) = crate::runner::native_stream_case::qualify(
        (
            "footer-config-order",
            &[
                "345efec64c7b95a05ce5cf96df0dde66a158835c",
                "GCode.cpp:4882-4935 non-BBL config then filament/compatible statistics",
                "GCode.cpp:4285-4300 distinct BBL header config location",
            ],
        ),
        label,
        project,
        sha,
        false,
    );
    assert!(
        reference.native.find("; CONFIG_BLOCK_START\n").unwrap()
            > reference.native.find("; EXECUTABLE_BLOCK_END\n").unwrap()
    );
    let native_tails = post_config_tails(&reference.native);
    assert_eq!(native_tails.len(), 1);
    let native = native_tails[0];
    std::fs::write(reference.root.join("native.post-config.gcode"), native).unwrap();
    let api = ares_core::slice_project_plate_validated(
        project,
        ares_core::GenerationMetadata::deterministic(2026, 10, 31, 0, 0, 0),
        None,
        &ares_core::ConfigOverrides::default(),
        ares_core::SlicerContext::new(0),
    )
    .await
    .unwrap();
    std::fs::write(reference.root.join("ares.api.gcode"), &api).unwrap();
    let api_tails = post_config_tails(std::str::from_utf8(&api).unwrap());
    assert_eq!(api_tails.len(), 1);
    let actual = api_tails[0];
    std::fs::write(reference.root.join("ares.post-config.gcode"), actual).unwrap();
    std::fs::write(
        reference.root.join("post-config-selection.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "native_config_end_ordinal": 0,
            "api_config_end_ordinal": 0,
            "native_config_end_markers": native_tails.len(),
            "api_config_end_markers": api_tails.len(),
            "native_config_after_executable": true,
            "all_bytes_after_marker_through_eof_included": true,
        }))
        .unwrap(),
    )
    .unwrap();
    assert_eq!(
        actual, native,
        "Complete post-CONFIG_END tail ordinal0 through EOF, including all statistics and blank lines, must match runtime Native: {:?}",
        reference.root,
    );
}

fn post_config_tails(gcode: &str) -> Vec<&str> {
    const END: &str = "; CONFIG_BLOCK_END\n";
    gcode
        .match_indices(END)
        .map(|(offset, _)| &gcode[offset + END.len()..])
        .collect()
}
