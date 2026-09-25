use super::{compare, document, island, lifecycle_island};

#[test]
fn sparse_infill_feed_edit_is_rejected() {
    // Unchained pairing must not ignore feed: a 10x feed on an
    // identical Sparse infill segment is a real speed divergence.
    let expected_island = island(0, 1, 1_200).replace("Inner wall", "Sparse infill");
    let actual_island = expected_island.replace("E.1 F1200", "E.1 F12000");
    let expected = document("1m", "1m", "10s", "2.00", &[&expected_island]);
    let actual = document("1m", "1m", "10s", "2.00", &[&actual_island]);

    compare(expected.as_bytes(), actual.as_bytes()).unwrap_err();
}

#[test]
fn sparse_infill_segment_rotation_is_rejected() {
    // A segment rotated 90° shares one endpoint and its E/length, but
    // no unmatched counterpart covers its far endpoint.
    let expected_island = island(0, 1, 1_200).replace("Inner wall", "Sparse infill");
    let actual_island = expected_island.replace("G1 X1 Y0 E.1", "G1 X0 Y1 E.1");
    let expected = document("1m", "1m", "10s", "2.00", &[&expected_island]);
    let actual = document("1m", "1m", "10s", "2.00", &[&actual_island]);

    compare(expected.as_bytes(), actual.as_bytes()).unwrap_err();
}

#[test]
fn reversed_degenerate_wall_loop_is_rejected() {
    // Degenerate (E<=0.02) leftovers may not bypass the wall reversal
    // guard: every edge must sit on a same-direction counterpart.
    let expected_island = "G1 X0 Y0 F6000\nM204 S5000\n; FEATURE: Inner wall\n; LINE_WIDTH: 0.45\nG1 X1 Y0 E.01 F1200\nG1 X1 Y1 E.01 F1200\nG1 X0 Y1 E.01 F1200\nG1 X0 Y0 E.01 F1200\n";
    let actual_island = "G1 X0 Y0 F6000\nM204 S5000\n; FEATURE: Inner wall\n; LINE_WIDTH: 0.45\nG1 X0 Y1 E.01 F1200\nG1 X1 Y1 E.01 F1200\nG1 X1 Y0 E.01 F1200\nG1 X0 Y0 E.01 F1200\n";
    let expected = document("1m", "1m", "10s", "2.00", &[expected_island]);
    let actual = document("1m", "1m", "10s", "2.00", &[actual_island]);

    compare(expected.as_bytes(), actual.as_bytes()).unwrap_err();
}

#[test]
fn tiny_legitimate_split_is_accepted() {
    // A 1µm quantization split of one segment (identical total E and
    // flow) reconciles against the matched larger piece.
    let expected_island = island(0, 1, 1_200);
    let actual_island = expected_island.replace(
        "G1 X1 Y0 E.1 F1200",
        "G1 X.001 Y0 E.0001 F1200\nG1 X1 Y0 E.0999 F1200",
    );
    let expected = document("1m", "1m", "10s", "2.00", &[&expected_island]);
    let actual = document("1m", "1m", "10s", "2.00", &[&actual_island]);

    compare(expected.as_bytes(), actual.as_bytes()).unwrap();
}

#[test]
fn duplicate_lifecycle_island_is_rejected() {
    // An extra retract/wipe island unbalances the extrusion pool even
    // when the opposite unmatched pool is empty.
    let expected_island = lifecycle_island(0, 1, ".1");
    let actual_island = format!(
        "{expected_island}G1 X1 Y0 F6000\nG1 E.4 F1800\nG1 E-.1 F1800\n; WIPE_START\nG1 X0 Y0 E-.05 F1200\n; WIPE_END\n"
    );
    let expected = document("1m", "1m", "10s", "2.00", &[&expected_island]);
    let actual = document("1m", "1m", "10s", "2.00", &[&actual_island]);

    compare(expected.as_bytes(), actual.as_bytes()).unwrap_err();
}

#[test]
fn hidden_extrusion_inside_wipe_island_is_rejected() {
    // An `E100` extruder move inside a novel island tips the extrusion
    // balance even though wipe-only extrusion stays under the cap.
    let expected_island = lifecycle_island(0, 1, ".1");
    let actual_island =
        format!("{expected_island}G1 E100 F1800\n; WIPE_START\nG1 X1 Y0 E-.05 F1200\n; WIPE_END\n");
    let expected = document("1m", "1m", "10s", "2.00", &[&expected_island]);
    let actual = document("1m", "1m", "10s", "2.00", &[&actual_island]);

    compare(expected.as_bytes(), actual.as_bytes()).unwrap_err();
}

#[test]
fn moved_m107_changes_deposition_fan_state() {
    // M107 must update parser fan state: moving it past the island
    // leaves the fan ON during extrusion — a real machine difference.
    let expected_island = format!("M106 S255\nM107\n{}", island(0, 1, 1_200));
    let actual_island = format!("M106 S255\n{}M107\n", island(0, 1, 1_200));
    let expected = document("1m", "1m", "10s", "2.00", &[&expected_island]);
    let actual = document("1m", "1m", "10s", "2.00", &[&actual_island]);

    compare(expected.as_bytes(), actual.as_bytes()).unwrap_err();
}

#[test]
fn plane_restoration_after_g18_is_rejected_when_missing() {
    // G18 leaves XY mode; a following G17 is NOT redundant and must be
    // preserved by control canonicalization.
    let body = "G1 X0 Y0 F6000\nM204 S5000\n; FEATURE: Inner wall\n; LINE_WIDTH: 0.45\nG3 X2 Y0 I1 J0 E.1 F1200\n";
    let expected = format!("G17\nG18\nG17\n{body}");
    let actual = format!("G17\nG18\n{body}");
    let expected = document("1m", "1m", "10s", "2.00", &[&expected]);
    let actual = document("1m", "1m", "10s", "2.00", &[&actual]);

    compare(expected.as_bytes(), actual.as_bytes()).unwrap_err();
}

// Borrowed-travel-acceleration (inserting `M204 S5000` before a hop so
// it borrows the deposition class) stays ACCEPTED: real layer-36 data
// reroutes a hop along the same path with the borrowed class, so the
// comparator cannot distinguish the two. Documented residual.

#[test]
fn small_major_arc_center_drift_is_rejected() {
    // A near-straight-scaled bound: radius ~50 allows only ~0.1mm of
    // center drift, not 0.5mm.
    let expected_island = "G1 X0 Y0 F6000\nM204 S5000\n; FEATURE: Inner wall\n; LINE_WIDTH: 0.45\nG2 X1 Y0 I.5 J50 E10 F1200\n";
    let actual_island = expected_island.replace("I.5 J50", "I.5 J50.5");
    let expected = document("1m", "1m", "10s", "2.00", &[expected_island]);
    let actual = document("1m", "1m", "10s", "2.00", &[&actual_island]);

    compare(expected.as_bytes(), actual.as_bytes()).unwrap_err();
}
