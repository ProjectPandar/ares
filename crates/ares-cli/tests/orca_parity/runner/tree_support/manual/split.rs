use super::{
    OrcaRunner, has_support_extrusion, overhang_case, paint_canopy, parity, slice_painted,
};

#[test]
fn manual_slim_split_painting_at_shared_edge_preserves_complete_output() {
    let label = "tree-support/manual/slim/split-painted-shared-edge";
    let Some(baseline) = overhang_case(label, "tree_slim", &[("support_type", "tree(manual)")])
    else {
        return;
    };
    assert!(!has_support_extrusion(
        std::str::from_utf8(&baseline.reference).unwrap()
    ));
    let runner = OrcaRunner::from_env().unwrap().unwrap();
    let leaf = slice_painted(
        &runner,
        paint_canopy(&baseline.project, &["4", "4"]),
        "leaf-control",
    );
    // Native nibble order: split three edges, then reverse-order child states
    // NONE/ENFORCER/NONE/ENFORCER. The adjacent root stays an ENFORCER leaf.
    let split = slice_painted(
        &runner,
        paint_canopy(&baseline.project, &["40403", "4"]),
        "split-painted",
    );
    let leaf_gcode = std::str::from_utf8(&leaf.reference).unwrap();
    let split_gcode = std::str::from_utf8(&split.reference).unwrap();
    assert!(has_support_extrusion(leaf_gcode));
    assert!(has_support_extrusion(split_gcode));
    assert_ne!(support_moves(leaf_gcode), support_moves(split_gcode));
    let outcome = parity::compare_case(&split);
    assert_eq!(outcome.status, "PASS", "{}", outcome.detail);
}

#[test]
fn manual_slim_nested_painting_on_both_shared_edge_faces_preserves_complete_output() {
    let label = "tree-support/manual/slim/nested-painting-both-shared-edge-faces";
    let Some(baseline) = overhang_case(label, "tree_slim", &[("support_type", "tree(manual)")])
    else {
        return;
    };
    assert!(!has_support_extrusion(
        std::str::from_utf8(&baseline.reference).unwrap()
    ));
    let runner = OrcaRunner::from_env().unwrap().unwrap();
    let control = slice_painted(
        &runner,
        paint_canopy(&baseline.project, &["40403", "4"]),
        "one-level-control",
    );
    // Both roots and their child0 split all edges; child0 neighbors share the
    // half-edge at vertex9, so their nested splits reuse its quarter point.
    let nested = slice_painted(
        &runner,
        paint_canopy(&baseline.project, &["404030403", "444434443"]),
        "nested-painted",
    );
    let control_gcode = std::str::from_utf8(&control.reference).unwrap();
    let nested_gcode = std::str::from_utf8(&nested.reference).unwrap();
    assert!(has_support_extrusion(control_gcode));
    assert!(has_support_extrusion(nested_gcode));
    assert_ne!(support_moves(control_gcode), support_moves(nested_gcode));
    let outcome = parity::compare_case(&nested);
    assert_eq!(outcome.status, "PASS", "{}", outcome.detail);
}

pub(super) fn support_moves(gcode: &str) -> Vec<[Option<&str>; 2]> {
    let mut support = false;
    let mut relative_e = false;
    gcode
        .lines()
        .filter_map(|line| {
            let instruction = line.split(';').next().unwrap();
            let words = instruction.split_ascii_whitespace();
            match words.clone().next() {
                Some("M83") => relative_e = true,
                Some("M82") => relative_e = false,
                _ => {}
            }
            if let Some(role) = line.strip_prefix(";TYPE:") {
                support = matches!(role, "Support" | "Support interface");
            }
            if !(relative_e && support && instruction.starts_with("G1 ")) {
                return None;
            }
            let coordinates =
                ['X', 'Y'].map(|axis| words.clone().find(|word| word.starts_with(axis)));
            let positive_e = words.clone().any(|word| {
                word.strip_prefix('E')
                    .is_some_and(|value| value.parse::<f64>().unwrap() > 0.0)
            });
            (positive_e && coordinates.iter().any(Option::is_some)).then_some(coordinates)
        })
        .collect()
}
