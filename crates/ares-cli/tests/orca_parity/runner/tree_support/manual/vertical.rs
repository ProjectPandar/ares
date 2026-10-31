use super::split::support_moves;
use super::{OrcaRunner, has_support_extrusion, overhang_case, paint_faces, parity, slice_painted};

#[test]
fn manual_slim_nested_vertical_painting_preserves_complete_output() {
    let Some(baseline) = overhang_case(
        "tree-support/manual/slim/nested-vertical-painting",
        "tree_slim",
        &[("support_type", "tree(manual)")],
    ) else {
        return;
    };
    assert!(!has_support_extrusion(
        std::str::from_utf8(&baseline.reference).unwrap()
    ));
    let runner = OrcaRunner::from_env().unwrap().unwrap();
    let control = slice_painted(
        &runner,
        paint_faces(&baseline.project, &["40403", "4"], |points| {
            points.iter().all(|point| point[0] == 15.0)
        }),
        "one-level-vertical-control",
    );
    let nested = slice_painted(
        &runner,
        paint_faces(&baseline.project, &["404030403", "444434443"], |points| {
            points.iter().all(|point| point[0] == 15.0)
        }),
        "nested-vertical-painted",
    );
    let control_gcode = std::str::from_utf8(&control.reference).unwrap();
    let nested_gcode = std::str::from_utf8(&nested.reference).unwrap();
    assert!(has_support_extrusion(control_gcode));
    assert!(has_support_extrusion(nested_gcode));
    assert_ne!(support_moves(control_gcode), support_moves(nested_gcode));
    let outcome = parity::compare_case(&nested);
    assert_eq!(outcome.status, "PASS", "{}", outcome.detail);
}

#[test]
fn manual_slim_vertical_painting_creates_support_and_preserves_complete_output() {
    let Some(baseline) = overhang_case(
        "tree-support/manual/slim/vertical-painting",
        "tree_slim",
        &[("support_type", "tree(manual)")],
    ) else {
        return;
    };
    assert!(!has_support_extrusion(
        std::str::from_utf8(&baseline.reference).unwrap()
    ));
    let runner = OrcaRunner::from_env().unwrap().unwrap();
    let project = paint_faces(&baseline.project, &["4", "4"], |points| {
        points.iter().all(|point| point[0] == 15.0)
    });
    let painted = slice_painted(&runner, project, "vertical-painted-manual");
    assert!(has_support_extrusion(
        std::str::from_utf8(&painted.reference).unwrap()
    ));
    let outcome = parity::compare_case(&painted);
    assert_eq!(outcome.status, "PASS", "{}", outcome.detail);
}

#[test]
fn manual_slim_split_vertical_painting_changes_support_paths_and_preserves_complete_output() {
    let Some(baseline) = overhang_case(
        "tree-support/manual/slim/split-vertical-painting",
        "tree_slim",
        &[("support_type", "tree(manual)")],
    ) else {
        return;
    };
    assert!(!has_support_extrusion(
        std::str::from_utf8(&baseline.reference).unwrap()
    ));
    let runner = OrcaRunner::from_env().unwrap().unwrap();
    let leaf = slice_painted(
        &runner,
        paint_faces(&baseline.project, &["4", "4"], |points| {
            points.iter().all(|point| point[0] == 15.0)
        }),
        "vertical-leaf-control",
    );
    let split = slice_painted(
        &runner,
        paint_faces(&baseline.project, &["40403", "4"], |points| {
            points.iter().all(|point| point[0] == 15.0)
        }),
        "split-vertical-painted",
    );
    let leaf_gcode = std::str::from_utf8(&leaf.reference).unwrap();
    let split_gcode = std::str::from_utf8(&split.reference).unwrap();
    assert!(has_support_extrusion(leaf_gcode));
    assert!(has_support_extrusion(split_gcode));
    assert_ne!(support_moves(leaf_gcode), support_moves(split_gcode));
    let outcome = parity::compare_case(&split);
    assert_eq!(outcome.status, "PASS", "{}", outcome.detail);
}
