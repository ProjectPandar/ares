use super::tree_support_brim::support::{
    assert_support_material_metadata, empty_contours, finalize, layer, layer_paths, support_rect,
};
use crate::{Point2, PrintPath, PrintPathRole};
use serde_json::{Value, json};

#[test]
fn positive_wall_count_emits_closed_sheath_before_inset_base_lines() {
    let finalized = finalize(
        vec![layer(
            1,
            support_rect(PrintPathRole::SupportMaterial, 0.0, 0.0, 2.0, 2.0),
        )],
        json!({
            "tree_support_wall_count": 1,
            "support_base_pattern_spacing": 0.0
        }),
        empty_contours(2),
    );
    let paths = finalized[0].paths();

    assert_eq!(paths.len(), 6);
    assert_eq!(paths[0].role(), PrintPathRole::SupportMaterial);
    assert!(paths[0].is_closed());
    assert_eq!(
        paths[0].points(),
        [
            Point2::new(0.0, 0.0),
            Point2::new(2.0, 0.0),
            Point2::new(2.0, 2.0),
            Point2::new(0.0, 2.0)
        ]
    );

    assert_support_lines(
        &paths[1..],
        &[
            [Point2::new(0.16, 0.16), Point2::new(1.84, 0.16)],
            [Point2::new(0.16, 0.56), Point2::new(1.84, 0.56)],
            [Point2::new(0.16, 0.96), Point2::new(1.84, 0.96)],
            [Point2::new(0.16, 1.36), Point2::new(1.84, 1.36)],
            [Point2::new(0.16, 1.76), Point2::new(1.84, 1.76)],
        ],
    );
}

#[test]
fn wall_count_two_matches_wall_count_one() {
    let one = finalize_with_options(json!({
        "tree_support_wall_count": 1,
        "support_base_pattern_spacing": 0.0
    }));
    let two = finalize_with_options(json!({
        "tree_support_wall_count": 2,
        "support_base_pattern_spacing": 0.0
    }));

    assert_eq!(two, one);
}

#[test]
fn zero_or_omitted_wall_count_preserves_current_base_output() {
    let omitted = finalize_with_options(json!({
        "support_base_pattern_spacing": 0.0
    }));
    let zero = finalize_with_options(json!({
        "tree_support_wall_count": 0,
        "support_base_pattern_spacing": 0.0
    }));

    assert_eq!(zero, omitted);
    assert_eq!(omitted[0].paths().len(), 6);
    assert!(omitted[0].paths().iter().all(|path| !path.is_closed()));
}

#[test]
fn rectilinear_grid_generates_both_families_from_inset_bounds() {
    let finalized = finalize(
        vec![layer(
            1,
            support_rect(PrintPathRole::SupportMaterial, 0.0, 0.0, 2.0, 1.0),
        )],
        json!({
            "tree_support_wall_count": 1,
            "support_base_pattern": "rectilinear-grid",
            "support_base_pattern_spacing": 0.0,
            "support_remove_small_overhang": false
        }),
        empty_contours(2),
    );
    let paths = finalized[0].paths();

    assert!(paths[0].is_closed());
    assert_eq!(
        paths[1].points(),
        [Point2::new(0.16, 0.16), Point2::new(1.84, 0.16)]
    );
    assert_eq!(
        paths[3].points(),
        [Point2::new(0.16, 0.16), Point2::new(0.16, 0.84)]
    );
    assert_eq!(
        paths.last().unwrap().points(),
        [Point2::new(1.76, 0.16), Point2::new(1.76, 0.84)]
    );
}

#[test]
fn collapsed_inset_emits_only_sheath_loop() {
    let finalized = finalize(
        vec![layer(
            1,
            support_rect(PrintPathRole::SupportMaterial, 0.0, 0.0, 0.2, 0.2),
        )],
        json!({
            "tree_support_wall_count": 1,
            "support_base_pattern_spacing": 0.0,
            "support_remove_small_overhang": false
        }),
        empty_contours(2),
    );

    assert_eq!(finalized[0].paths().len(), 1);
    assert!(finalized[0].paths()[0].is_closed());
}

#[test]
fn sheath_and_infill_preserve_support_material_metadata() {
    let support = support_rect(PrintPathRole::SupportMaterial, 0.0, 0.0, 2.0, 2.0)
        .with_extrusion_role(PrintPathRole::SupportMaterial)
        .with_effective_layer_height_mm(0.16)
        .with_effective_line_width_mm(Some(0.44))
        .with_unsupported_span_mm(Some(1.2))
        .with_seam_gap_mm(0.08);

    let finalized = finalize(
        vec![layer(1, support)],
        json!({
            "tree_support_wall_count": 1,
            "support_base_pattern_spacing": 0.0
        }),
        empty_contours(2),
    );

    for path in finalized[0].paths() {
        assert_support_material_metadata(path);
    }
}

#[test]
fn non_target_paths_are_unchanged_by_wall_sheath() {
    let open_support =
        support_rect(PrintPathRole::SupportMaterial, 0.0, 0.0, 2.0, 2.0).with_closed(false);
    let triangle = PrintPath::new(
        PrintPathRole::SupportMaterial,
        vec![
            Point2::new(0.0, 0.0),
            Point2::new(2.0, 0.0),
            Point2::new(1.0, 2.0),
        ],
    )
    .unwrap()
    .with_closed(true);
    let interface = support_rect(PrintPathRole::SupportMaterialInterface, 0.0, 0.0, 2.0, 2.0)
        .with_closed(false);
    let solid = support_rect(PrintPathRole::SolidInfill, 0.0, 0.0, 2.0, 2.0);

    let finalized = finalize(
        vec![layer_paths(
            1,
            vec![
                open_support.clone(),
                triangle.clone(),
                interface.clone(),
                solid.clone(),
            ],
        )],
        json!({ "tree_support_wall_count": 1 }),
        empty_contours(2),
    );

    assert_eq!(
        finalized[0].paths(),
        [open_support, triangle, interface, solid]
    );
}

#[test]
fn zero_top_interface_layers_converted_support_material_receives_sheath() {
    let finalized = finalize(
        vec![layer(
            1,
            support_rect(PrintPathRole::SupportMaterialInterface, 0.0, 0.0, 2.0, 2.0)
                .with_extrusion_role(PrintPathRole::SupportMaterialInterface),
        )],
        json!({
            "tree_support_wall_count": 1,
            "support_interface_top_layers": 0,
            "support_base_pattern_spacing": 0.0
        }),
        empty_contours(2),
    );

    assert_eq!(
        finalized[0].paths()[0].role(),
        PrintPathRole::SupportMaterial
    );
    assert_eq!(finalized[0].paths()[0].extrusion_role(), None);
    assert!(finalized[0].paths()[0].is_closed());
    assert_eq!(finalized[0].paths().len(), 6);
}

fn finalize_with_options(extra: Value) -> Vec<crate::LayerPrintPaths> {
    finalize(
        vec![layer(
            1,
            support_rect(PrintPathRole::SupportMaterial, 0.0, 0.0, 2.0, 2.0),
        )],
        extra,
        empty_contours(2),
    )
}

fn assert_support_lines(paths: &[PrintPath], expected: &[[Point2; 2]]) {
    assert_eq!(paths.len(), expected.len());
    for (path, points) in paths.iter().zip(expected) {
        assert_eq!(path.role(), PrintPathRole::SupportMaterial);
        assert_eq!(path.points(), *points);
        assert!(!path.is_closed());
    }
}
