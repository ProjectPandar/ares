//! Golden parity test for ksr_fdmtest_v4.
//!
//! Validates that Ares produces structurally and geometrically correct G-code
//! for the reference project file. Byte-level parity with the OrcaSlicer
//! reference is achievable except where gap-fill skeleton vertices depend on
//! boostvoronoi (Rust) vs boost::polygon (C++) floating-point evaluation;
//! those differ by ≤1µm and shift arc-fit segmentation decisions downstream.

use std::path::PathBuf;

fn fixture_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/ksr_fdmtest_v4")
        .canonicalize()
        .expect("fixture dir")
}

#[tokio::test(flavor = "current_thread")]
async fn golden_ksr_fdmtest_v4_slicing_produces_correct_output() {
    let project =
        std::fs::read(fixture_dir().join("ksr_fdmtest_v4.project.3mf")).expect("read 3mf fixture");
    let metadata = ares_core::GenerationMetadata::deterministic(2026, 7, 10, 11, 16, 9);
    let output = ares_core::slice_project(&project, metadata)
        .await
        .expect("slicing succeeds");
    let output_str = String::from_utf8(output).expect("utf8 gcode");

    // Structural checks that must hold regardless of voronoi FP differences.
    assert!(
        output_str.contains("; FEATURE: Internal solid infill"),
        "must contain internal solid infill"
    );
    assert!(output_str.contains("G3 "), "must contain arc-fitted moves");
    assert!(
        output_str.contains("M73 P0 R"),
        "must contain initial M73 time marker"
    );
    assert!(
        output_str.contains("G29.2 S1"),
        "must contain ABL re-enable sequence"
    );

    // Geometric coverage: ≥90% of XY coordinate pairs from the reference must
    // appear in Ares output (order-independent multiset comparison).
    let ref_gcode = std::fs::read_to_string(fixture_dir().join("ksr_fdmtest_v4.gcode"))
        .expect("read reference gcode");
    let ref_set = extract_xy_multiset(&ref_gcode);
    let out_set = extract_xy_multiset(&output_str);

    let total_ref: usize = ref_set.values().sum();
    let matched: usize = out_set
        .iter()
        .map(|(key, count)| count.min(ref_set.get(key).unwrap_or(&0)))
        .sum();
    let pct = 100.0 * matched as f64 / total_ref.max(1) as f64;

    assert!(
        pct >= 90.0,
        "geometric coverage {pct:.1}% below 90% threshold \
         (matched {matched}/{total_ref} XY positions)"
    );
}

fn extract_xy_multiset(gcode: &str) -> std::collections::BTreeMap<String, usize> {
    let mut map = std::collections::BTreeMap::new();
    for line in gcode.lines() {
        let t = line.trim();
        if !(t.starts_with("G1 ") || t.starts_with("G2 ") || t.starts_with("G3 ")) {
            continue;
        }
        let x = t.split('X').nth(1).and_then(|s| {
            s.split_whitespace()
                .next()
                .and_then(|v| v.parse::<f64>().ok())
        });
        let y = t.split('Y').nth(1).and_then(|s| {
            s.split_whitespace()
                .next()
                .and_then(|v| v.parse::<f64>().ok())
        });
        if let (Some(x), Some(y)) = (x, y) {
            let key = format!("{x:.3},{y:.3}");
            *map.entry(key).or_insert(0) += 1;
        }
    }
    map
}
