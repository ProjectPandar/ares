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

#[test]
fn degenerate_feed_edit_is_rejected() {
    // A degenerate (E<=0.02) sliver at 10x feed is an edit, not a
    // quantization sliver: the feed is absent from the other side's
    // deposition vocabulary.
    let expected_island = "G1 X0 Y0 F6000\nM204 S5000\n; FEATURE: Inner wall\n; LINE_WIDTH: 0.45\nG1 X1 Y0 E.001 F1200\n";
    let actual_island = expected_island.replace("E.001 F1200", "E.001 F12000\nG1 F1200");
    let expected = document("1m", "1m", "10s", "2.00", &[expected_island]);
    let actual = document("1m", "1m", "10s", "2.00", &[&actual_island]);

    compare(expected.as_bytes(), actual.as_bytes()).unwrap_err();
}

#[test]
fn degenerate_z_hop_is_rejected() {
    // A sliver lifted to a different Z is not a same-path sliver.
    let expected_island = "G1 X0 Y0 F6000\nM204 S5000\n; FEATURE: Inner wall\n; LINE_WIDTH: 0.45\nG1 X1 Y0 E.001 F1200\n";
    let actual_island = expected_island.replace("G1 X1 Y0 E.001", "G1 X1 Y0 Z50 E.001");
    let expected = document("1m", "1m", "10s", "2.00", &[expected_island]);
    let actual = document("1m", "1m", "10s", "2.00", &[&actual_island]);

    compare(expected.as_bytes(), actual.as_bytes()).unwrap_err();
}

#[test]
fn sparse_infill_53deg_rotation_is_rejected() {
    // Rotated by 53° on a shared vertex: |dot| = 0.6 < 0.7 bound.
    let expected_island = island(0, 1, 1_200).replace("Inner wall", "Sparse infill");
    let actual_island = expected_island.replace("G1 X1 Y0 E.1", "G1 X.6 Y.8 E.1");
    let expected = document("1m", "1m", "10s", "2.00", &[&expected_island]);
    let actual = document("1m", "1m", "10s", "2.00", &[&actual_island]);

    compare(expected.as_bytes(), actual.as_bytes()).unwrap_err();
}

#[test]
fn translated_rotation_is_rejected() {
    // A 90° rotation translated by 6µm no longer shares a vertex
    // exactly, but 6µm is still inside the shared-vertex bound.
    let expected_island = island(0, 1, 1_200).replace("Inner wall", "Sparse infill");
    let actual_island = expected_island
        .replace("G1 X0 Y0 F6000", "G1 X.006 Y0 F6000")
        .replace("G1 X1 Y0 E.1", "G1 X.006 Y1 E.1");
    let expected = document("1m", "1m", "10s", "2.00", &[&expected_island]);
    let actual = document("1m", "1m", "10s", "2.00", &[&actual_island]);

    compare(expected.as_bytes(), actual.as_bytes()).unwrap_err();
}

#[test]
fn canceling_large_extruder_pair_is_rejected() {
    // E100 + E-100 nets to ~zero but deposits 200mm of extruder motion;
    // the per-island excursion cap rejects it.
    let expected_island = lifecycle_island(0, 1, ".1");
    let actual_island = format!(
        "{expected_island}G1 E100 F1800\nG1 E-100 F1800\n; WIPE_START\nG1 X1 Y0 E-.05 F1200\n; WIPE_END\n"
    );
    let expected = document("1m", "1m", "10s", "2.00", &[&expected_island]);
    let actual = document("1m", "1m", "10s", "2.00", &[&actual_island]);

    compare(expected.as_bytes(), actual.as_bytes()).unwrap_err();
}

#[test]
fn arc_center_drift_bound_is_contour_deviation() {
    // Mid-sweep contour deviation, not raw center distance: a 0.09mm
    // J shift on a radius-50 near-straight arc is caught.
    let expected_island = "G1 X0 Y0 F6000\nM204 S5000\n; FEATURE: Inner wall\n; LINE_WIDTH: 0.45\nG2 X1 Y0 I.5 J50 E10 F1200\n";
    let actual_island = expected_island.replace("I.5 J50", "I.5 J50.09");
    let expected = document("1m", "1m", "10s", "2.00", &[expected_island]);
    let actual = document("1m", "1m", "10s", "2.00", &[&actual_island]);

    compare(expected.as_bytes(), actual.as_bytes()).unwrap_err();
}

#[test]
fn arc_under_wrong_plane_is_rejected() {
    // The MotionRecord plane snapshot makes a G17 placed AFTER the arc
    // visible: the arc executes under G18, not G17.
    let body = "G1 X0 Y0 F6000\nM204 S5000\n; FEATURE: Inner wall\n; LINE_WIDTH: 0.45\nG3 X2 Y0 I1 J0 E.1 F1200\n";
    let expected = format!("G17\nG18\nG17\n{body}");
    let actual = format!("G17\nG18\n{body}G17\n");
    let expected = document("1m", "1m", "10s", "2.00", &[&expected]);
    let actual = document("1m", "1m", "10s", "2.00", &[&actual]);

    compare(expected.as_bytes(), actual.as_bytes()).unwrap_err();
}

// Borrowed-travel-acceleration (inserting `M204 S5000` before a hop so
// it borrows the deposition class) stays ACCEPTED: real layer-36 data
// reroutes a hop along the same path with the borrowed class, so the
// comparator cannot distinguish the two. Documented residual.

#[test]
fn sliver_with_foreign_machine_state_is_rejected() {
    // A sub-0.008mm sliver still has to carry a real deposition feed
    // and the same accel/fans/z — it is not a machine-state bypass.
    let expected_island = island(0, 1, 1_200).replace("G1 X1 Y0 E.1", "G1 X.12 Y0 E.004");
    let actual_island = expected_island.replace("F1200", "F12000");
    let expected = document("1m", "1m", "10s", "2.00", &[&expected_island]);
    let actual = document("1m", "1m", "10s", "2.00", &[&actual_island]);

    compare(expected.as_bytes(), actual.as_bytes()).unwrap_err();
}

#[test]
fn long_sliver_geometry_is_rejected() {
    // Extrusion under the sliver cap does not excuse a 100mm move.
    let expected_island = island(0, 1, 1_200).replace("G1 X1 Y0 E.1", "G1 X.12 Y0 E.004");
    let actual_island = expected_island.replace("G1 X.12 Y0", "G1 X100 Y0");
    let expected = document("1m", "1m", "10s", "2.00", &[&expected_island]);
    let actual = document("1m", "1m", "10s", "2.00", &[&actual_island]);

    compare(expected.as_bytes(), actual.as_bytes()).unwrap_err();
}

#[test]
fn degenerate_arc_center_swap_is_rejected() {
    // Degenerate containment reasons about arcs by curve geometry, not
    // endpoints: J0→J20 changes the sweep entirely.
    let expected_island = "G1 X0 Y0 F6000\nM204 S5000\n; FEATURE: Inner wall\n; LINE_WIDTH: 0.45\nG2 X.2 Y0 I.1 J0 E.01 F1200\n";
    let actual_island = expected_island.replace("I.1 J0", "I.1 J20");
    let expected = document("1m", "1m", "10s", "2.00", &[expected_island]);
    let actual = document("1m", "1m", "10s", "2.00", &[&actual_island]);

    compare(expected.as_bytes(), actual.as_bytes()).unwrap_err();
}

#[test]
fn multi_turn_arc_center_drift_is_rejected() {
    // P-turn arcs sample in angle space: quarter-fraction sampling
    // aliases whole revolutions, so center drift is measured on the
    // shared first turn.
    let expected_island = "G1 X0 Y0 F6000\nM204 S5000\n; FEATURE: Inner wall\n; LINE_WIDTH: 0.45\nG3 X1 Y0 I.5 J50 P4 E10 F1200\n";
    let actual_island = expected_island.replace("I.5 J50", "I.5 J50.5");
    let expected = document("1m", "1m", "10s", "2.00", &[expected_island]);
    let actual = document("1m", "1m", "10s", "2.00", &[&actual_island]);

    compare(expected.as_bytes(), actual.as_bytes()).unwrap_err();
}

#[test]
fn sparse_infill_plane_swap_is_rejected() {
    // Plane mismatch rejects on every fallback path, not only primary
    // matching — a leftover under G18 never pairs with a G17
    // counterpart.
    let body = "G1 X0 Y0 F6000\nM204 S5000\n; FEATURE: Sparse infill\n; LINE_WIDTH: 0.45\nG3 X2 Y0 I1 J0 E.1 F1200\n";
    let expected = format!("G17\nG18\nG17\n{body}");
    let actual = format!("G17\nG18\n{body}G17\n");
    let expected = document("1m", "1m", "10s", "2.00", &[&expected]);
    let actual = document("1m", "1m", "10s", "2.00", &[&actual]);

    compare(expected.as_bytes(), actual.as_bytes()).unwrap_err();
}

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

#[test]
fn same_endpoints_huge_arc_rewrite_is_rejected() {
    // Internal solid infill arc with identical endpoints/extrusion but
    // a rewritten center (0.05mm → 39.5mm radius) is an edit, not a
    // restitch: same-command arc leftovers need a sane radius/chord.
    let expected_island = "G1 X0 Y0 F6000\nM204 S5000\n; FEATURE: Internal solid infill\n; LINE_WIDTH: 0.45\nG2 X.088 Y.079 I-.023 J.041 E.007 F1200\n";
    let actual_island = expected_island.replace("I-.023 J.041", "I-39.537 J2.542");
    let expected = document("1m", "1m", "10s", "2.00", &[expected_island]);
    let actual = document("1m", "1m", "10s", "2.00", &[&actual_island]);

    compare(expected.as_bytes(), actual.as_bytes()).unwrap_err();
}

#[test]
fn arc_deposited_at_wrong_z_is_rejected() {
    // Reordering vertical travels around a sparse-infill arc must not
    // reassign its Z: the arc-containment clause checks Z too.
    let body = "G1 X0 Y0 F6000\nM204 S5000\n; FEATURE: Sparse infill\n; LINE_WIDTH: 0.45\nG3 X2 Y0 I1 J0 E.1 F1200\n";
    let expected = format!("G1 Z.4\nG1 Z.2\n{body}");
    let actual = format!("G1 Z.2\nG1 Z.4\n{body}");
    let expected = document("1m", "1m", "10s", "2.00", &[&expected]);
    let actual = document("1m", "1m", "10s", "2.00", &[&actual]);

    compare(expected.as_bytes(), actual.as_bytes()).unwrap_err();
}

#[test]
fn rotated_unit_strand_is_rejected() {
    // A 1mm strand rotated 90° plus a 30µm translation is not a
    // boundary restitch even under the direction exemption.
    let expected_island = "G1 X0 Y0 F6000\nM204 S5000\n; FEATURE: Sparse infill\n; LINE_WIDTH: 0.45\nG1 X1 Y0 E.1 F1200\n";
    let actual_island = "G1 X0 Y0 F6000\nM204 S5000\n; FEATURE: Sparse infill\n; LINE_WIDTH: 0.45\nG1 X.03 Y0 F6000\nG1 X.03 Y1 E.1 F1200\n";
    let expected = document("1m", "1m", "10s", "2.00", &[expected_island]);
    let actual = document("1m", "1m", "10s", "2.00", &[actual_island]);

    compare(expected.as_bytes(), actual.as_bytes()).unwrap_err();
}

#[test]
fn rotated_long_strand_is_rejected() {
    // A 100mm strand rotated 90° with a 1.5mm offset is far outside
    // every leftover path-distance bound.
    let expected_island = "G1 X0 Y0 F6000\nM204 S5000\n; FEATURE: Sparse infill\n; LINE_WIDTH: 0.45\nG1 X100 Y0 E3 F1200\n";
    let actual_island = "G1 X0 Y0 F6000\nM204 S5000\n; FEATURE: Sparse infill\n; LINE_WIDTH: 0.45\nG1 X1.5 Y0 F6000\nG1 X1.5 Y100 E3 F1200\n";
    let expected = document("1m", "1m", "10s", "2.00", &[expected_island]);
    let actual = document("1m", "1m", "10s", "2.00", &[actual_island]);

    compare(expected.as_bytes(), actual.as_bytes()).unwrap_err();
}

#[test]
fn multi_turn_arc_turn_count_is_compared() {
    // Adding P1 to a degenerate arc changes the deposited sweep;
    // turn counts are part of arc identity on every path.
    let expected_island = "G1 X0 Y0 F6000\nM204 S5000\n; FEATURE: Inner wall\n; LINE_WIDTH: 0.45\nG3 X.2 Y0 I.1 J0 E.01 F1200\n";
    let actual_island = expected_island.replace("I.1 J0 E.01", "I.1 J0 P1 E.01");
    let expected = document("1m", "1m", "10s", "2.00", &[expected_island]);
    let actual = document("1m", "1m", "10s", "2.00", &[&actual_island]);

    compare(expected.as_bytes(), actual.as_bytes()).unwrap_err();
}

#[test]
fn nearly_straight_arc_endpoint_drift_is_accepted() {
    // Positive case from the review: a 0.002mm endpoint change on a
    // ~287mm-radius arc is within the contour tolerance.
    let expected_island = "G1 X0 Y0 F6000\nM204 S5000\n; FEATURE: Inner wall\n; LINE_WIDTH: 0.45\nG3 X1 Y0 I.5 J287 E.1 F1200\n";
    let actual_island = expected_island.replace("X1 Y0 I.5 J287", "X1 Y-.002 I1 J287");
    let expected = document("1m", "1m", "10s", "2.00", &[expected_island]);
    let actual = document("1m", "1m", "10s", "2.00", &[&actual_island]);

    compare(expected.as_bytes(), actual.as_bytes()).unwrap();
}
