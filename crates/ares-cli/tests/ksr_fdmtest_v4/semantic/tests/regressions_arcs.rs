use super::{compare, document};

#[test]
fn same_endpoints_huge_arc_rewrite_is_rejected() {
    // Internal solid infill arc with identical endpoints/extrusion but
    // a rewritten center (0.047mm → 19.8mm radius, consistent endpoint
    // radii) is an edit, not a restitch: arc leftovers need sane
    // radius/chord AND swept/chord bounds.
    let expected_island = "G1 X0 Y0 F6000\nM204 S5000\n; FEATURE: Internal solid infill\n; LINE_WIDTH: 0.45\nG2 X.005 Y.079 I-.023 J.041 E.007 F1200\n";
    let actual_island = expected_island.replace("I-.023 J.041", "I-19.787 J1.292");
    let expected = document("1m", "1m", "10s", "2.00", &[expected_island]);
    let actual = document("1m", "1m", "10s", "2.00", &[&actual_island]);

    compare(expected.as_bytes(), actual.as_bytes()).unwrap_err();
}

#[test]
fn arc_deposited_at_wrong_z_is_rejected() {
    // Reordering vertical travels around a sparse-infill arc leaves
    // both depositions at Z.2 — the comparator rejects via the
    // differing travel profiles. Deposition-height drift itself is
    // covered by arc_deposition_end_z_change_is_rejected.
    let body = "G1 X0 Y0 Z.2 F6000\nM204 S5000\n; FEATURE: Sparse infill\n; LINE_WIDTH: 0.45\nG3 X2 Y0 I1 J0 E.1 F1200\n";
    let expected = format!("G1 Z.4 F6000\nG1 Z.2 F6000\n{body}");
    let actual = format!("G1 Z.2 F6000\nG1 Z.4 F6000\n{body}");
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

#[test]
fn same_endpoints_long_sweep_arc_rewrite_is_rejected() {
    // A consistent-radius rewrite that sweeps a full ~124mm circle on
    // a 0.09mm chord (cross-command G3 form too) is an edit, not a
    // restitch — arc sanity also bounds swept/chord.
    let expected_island = "G1 X0 Y0 F6000\nM204 S5000\n; FEATURE: Internal solid infill\n; LINE_WIDTH: 0.45\nG2 X.005 Y.079 I-.023 J.041 E.007 F1200\n";
    let actual_island = expected_island.replace(
        "G2 X.005 Y.079 I-.023 J.041",
        "G3 X.005 Y.079 I19.792 J-1.213",
    );
    let expected = document("1m", "1m", "10s", "2.00", &[expected_island]);
    let actual = document("1m", "1m", "10s", "2.00", &[&actual_island]);

    compare(expected.as_bytes(), actual.as_bytes()).unwrap_err();
}

#[test]
fn same_path_extrusion_rewrite_is_rejected() {
    // Identical geometry with ~6x extrusion: coverage without flow
    // reconciliation would have swallowed this.
    let expected_island = "G1 X0 Y0 F6000\nM204 S5000\n; FEATURE: Internal solid infill\n; LINE_WIDTH: 0.45\nG1 X1.341 Y0 E.041 F1200\n";
    let actual_island = expected_island.replace("E.041", "E.241");
    let expected = document("1m", "1m", "10s", "2.00", &[expected_island]);
    let actual = document("1m", "1m", "10s", "2.00", &[&actual_island]);

    compare(expected.as_bytes(), actual.as_bytes()).unwrap_err();
}

#[test]
fn arc_deposition_end_z_change_is_rejected() {
    // An arc whose endpoint carries a Z move deposits at a different
    // height; paired checks the full Z profile now.
    let expected_island = "G1 X0 Y0 Z.2 F6000\nM204 S5000\n; FEATURE: Sparse infill\n; LINE_WIDTH: 0.45\nG3 X2 Y0 I1 J0 E.1 F1200\n";
    let actual_island = expected_island.replace("G3 X2 Y0 I1 J0", "G3 X2 Y0 Z.4 I1 J0");
    let expected = document("1m", "1m", "10s", "2.00", &[expected_island]);
    let actual = document("1m", "1m", "10s", "2.00", &[&actual_island]);

    compare(expected.as_bytes(), actual.as_bytes()).unwrap_err();
}

#[test]
fn long_perpendicular_strand_sharing_start_is_rejected() {
    // An 80mm perpendicular replacement is not a corner cap: caps
    // join a short tip (<=2mm) to a longer leg.
    let expected_island = "G1 X0 Y0 F6000\nM204 S5000\n; FEATURE: Sparse infill\n; LINE_WIDTH: 0.45\nG1 X100 Y0 E3 F1200\n";
    let actual_island = "G1 X0 Y0 F6000\nM204 S5000\n; FEATURE: Sparse infill\n; LINE_WIDTH: 0.45\nG1 X0 Y80 E3 F1200\n";
    let expected = document("1m", "1m", "10s", "2.00", &[expected_island]);
    let actual = document("1m", "1m", "10s", "2.00", &[actual_island]);

    compare(expected.as_bytes(), actual.as_bytes()).unwrap_err();
}

#[test]
fn multi_turn_arc_under_coverage_bound_is_rejected() {
    // Coverage measures point-to-path on the swept circle and
    // delegates turn-aware circle comparison: J1->J1.5 on a P4 arc
    // shifts the contour 0.5mm on every turn.
    let expected_island = "G1 X0 Y0 Z.2 F6000\nM204 S5000\n; FEATURE: Sparse infill\n; LINE_WIDTH: 0.45\nG3 X.002 Y0 I.001 J1 P4 E.8 F1200\n";
    let actual_island = expected_island.replace("I.001 J1 P4", "I.001 J1.5 P4");
    let expected = document("1m", "1m", "10s", "2.00", &[expected_island]);
    let actual = document("1m", "1m", "10s", "2.00", &[&actual_island]);

    compare(expected.as_bytes(), actual.as_bytes()).unwrap_err();

    // P4 -> P8 doubles the deposited sweep on the same circle.
    let actual_extra_turns = expected_island.replace("P4", "P8");
    let actual = document("1m", "1m", "10s", "2.00", &[&actual_extra_turns]);
    compare(expected.as_bytes(), actual.as_bytes()).unwrap_err();
}

#[test]
fn same_chord_arc_contour_rewrite_is_rejected() {
    // Same command, same chord, same E — but the redrawn center
    // moves the bulge ~4.6mm. Swept/chord stays under 30x, so only
    // the same-chord contour bound rejects it.
    let expected_island = "G1 X50 Y50 Z.2 F6000\nM204 S5000\n; FEATURE: Internal solid infill\n; LINE_WIDTH: 0.45\nG2 X60 Y50 I5 J0 E.5 F1200\n";
    let actual_island = expected_island.replace("I5 J0", "I5 J30");
    let expected = document("1m", "1m", "10s", "2.00", &[expected_island]);
    let actual = document("1m", "1m", "10s", "2.00", &[&actual_island]);

    compare(expected.as_bytes(), actual.as_bytes()).unwrap_err();
}

#[test]
fn arc_swept_over_chord_rewrite_is_rejected() {
    // A 0.2mm chord redrawn into a ~13mm sweep (~65x per chord) is
    // beyond any real refit ratio (~25x).
    let expected_island = "G1 X168.435 Y143.818 Z.4 F6000\nM204 S5000\n; FEATURE: Internal solid infill\n; LINE_WIDTH: 0.45\nG2 X168.44 Y143.897 I-.023 J.041 E.007 F1200\n";
    let actual_island = expected_island.replace("I-.023 J.041", "I-.827 J.092");
    let expected = document("1m", "1m", "10s", "2.00", &[expected_island]);
    let actual = document("1m", "1m", "10s", "2.00", &[&actual_island]);

    compare(expected.as_bytes(), actual.as_bytes()).unwrap_err();
}

#[test]
fn strands_far_off_arc_circle_are_rejected() {
    // Strands ~8mm off the arc's extrapolated circle: the angular
    // slack on the point-to-arc distance is distance-scaled, not a
    // fixed 0.1 rad (which covered ~28mm at r287).
    let prefix = "M204 S5000\nG1 X0 Y0 Z.2 F6000\n; FEATURE: Sparse infill\n; LINE_WIDTH: 0.45\nG3 X1 Y0 I.5 J287 E.034 F1200\n";
    let expected_island = format!("{prefix}G1 X2 Y.003 F6000\nG1 X4 Y.021 E.068 F1200\n");
    let actual_island = format!("{prefix}G1 X10 Y.157 F6000\nG1 X12 Y.23 E.068 F1200\n");
    let expected = document("1m", "1m", "10s", "2.00", &[&expected_island]);
    let actual = document("1m", "1m", "10s", "2.00", &[&actual_island]);

    compare(expected.as_bytes(), actual.as_bytes()).unwrap_err();
}

#[test]
fn same_arc_extrusion_rewrite_is_rejected() {
    // An identical arc with ~30x the extrusion fails the containment
    // flow bound.
    let expected_island = "G1 X168.435 Y143.818 Z.4 F6000\nM204 S5000\n; FEATURE: Internal solid infill\n; LINE_WIDTH: 0.45\nG2 X168.44 Y143.897 I-.023 J.041 E.007 F1200\n";
    let actual_island = expected_island.replace("E.007", "E.207");
    let expected = document("1m", "1m", "10s", "2.00", &[expected_island]);
    let actual = document("1m", "1m", "10s", "2.00", &[&actual_island]);

    compare(expected.as_bytes(), actual.as_bytes()).unwrap_err();
}

#[test]
fn local_extrusion_redistribution_is_rejected() {
    // Moving +.02/-.02 across two unchanged strands is a ~48% local
    // flow drift — beyond the ~26% a real restitch needs.
    let island = |e1: &str, e2: &str| {
        format!(
            "G1 X167.461 Y144.255 Z.4 F6000\nM204 S10000\n; FEATURE: Internal solid infill\n; LINE_WIDTH: 0.42\nG1 X168.802 Y144.255 E{e1} F1200\nG1 X168.802 Y142.914 E{e2} F1200\n"
        )
    };
    let expected = document("1m", "1m", "10s", "2.00", &[&island(".04121", ".04121")]);
    let actual = document("1m", "1m", "10s", "2.00", &[&island(".06121", ".02121")]);

    compare(expected.as_bytes(), actual.as_bytes()).unwrap_err();
}

#[test]
fn exact_arc_split_is_accepted() {
    // A 270-degree arc redrawn as three 90-degree arcs: identical
    // swept geometry, extrusion conserved per swept length.
    let prefix = "G1 X50 Y50 Z.2 F6000\nM204 S5000\n; FEATURE: Sparse infill\n; LINE_WIDTH: 0.45\n";
    let expected_island = format!("{prefix}G3 X51 Y51 I1 J0 E.15 F1200\n");
    let actual_island = format!(
        "{prefix}G3 X51 Y49 I1 J0 E.05 F1200\nG3 X52 Y50 I0 J1 E.05 F1200\nG3 X51 Y51 I-1 J0 E.05 F1200\n"
    );
    let expected = document("1m", "1m", "10s", "2.00", &[&expected_island]);
    let actual = document("1m", "1m", "10s", "2.00", &[&actual_island]);

    compare(expected.as_bytes(), actual.as_bytes()).unwrap();
}

#[test]
fn arc_cap_and_rest_redraw_is_accepted() {
    // A 30mm arc redrawn as a 0.64mm cap plus the rest of the same
    // circle: both ends of the cap land near one endpoint of the big
    // arc — an adjacency, not a shared chord.
    let prefix =
        "G1 X50 Y50 Z.2 F6000\nM204 S5000\n; FEATURE: Internal solid infill\n; LINE_WIDTH: 0.45\n";
    let expected_island = format!("{prefix}G2 X80 Y80 I30 J0 E1.5 F1200\n");
    let actual_island = format!(
        "{prefix}G2 X50.9 Y50.9 I.45 J.45 E.06364 F1200\nG2 X80 Y80 I29.1 J0 E1.455 F1200\n"
    );
    let expected = document("1m", "1m", "10s", "2.00", &[&expected_island]);
    let actual = document("1m", "1m", "10s", "2.00", &[&actual_island]);

    compare(expected.as_bytes(), actual.as_bytes()).unwrap();
}
