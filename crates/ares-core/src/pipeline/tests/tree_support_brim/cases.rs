use super::support::{
    assert_support_material_bounds, assert_support_material_metadata, contains_exact_path,
    empty_contours, finalize, layer, layer_by_id, layer_paths, support_rect,
};
use crate::{Point2, PrintPath, PrintPathRole};
use serde_json::json;

#[test]
fn manual_tree_support_brim_expands_first_layer_support_material() {
    let finalized = finalize(
        vec![layer(
            0,
            support_rect(PrintPathRole::SupportMaterial, 0.0, 0.0, 2.0, 2.0),
        )],
        json!({
            "support_type": "tree(auto)",
            "tree_support_auto_brim": false,
            "tree_support_brim_width": 1.25
        }),
        empty_contours(1),
    );

    assert_support_material_bounds(&finalized[0], -1.25, -1.25, 3.25, 3.25);
}

#[test]
fn manual_tree_support_brim_expands_tree_manual_support_material() {
    let finalized = finalize(
        vec![layer(
            0,
            support_rect(PrintPathRole::SupportMaterial, 1.0, 1.0, 3.0, 3.0),
        )],
        json!({
            "support_type": "tree(manual)",
            "tree_support_auto_brim": false,
            "tree_support_brim_width": 0.5
        }),
        empty_contours(1),
    );

    assert_support_material_bounds(&finalized[0], 0.5, 0.5, 3.5, 3.5);
}

#[test]
fn manual_tree_support_brim_preserves_support_material_metadata_after_spacing() {
    let support = support_rect(PrintPathRole::SupportMaterial, 1.0, 1.0, 3.0, 3.0)
        .with_extrusion_role(PrintPathRole::SupportMaterial)
        .with_effective_layer_height_mm(0.16)
        .with_effective_line_width_mm(Some(0.44))
        .with_unsupported_span_mm(Some(1.2))
        .with_seam_gap_mm(0.08);

    let finalized = finalize(
        vec![layer(0, support)],
        json!({
            "support_type": "tree(auto)",
            "tree_support_auto_brim": false,
            "tree_support_brim_width": 0.5,
            "support_base_pattern_spacing": 0.1
        }),
        empty_contours(1),
    );

    let support_material_paths: Vec<_> = finalized[0]
        .paths()
        .iter()
        .filter(|path| path.role() == PrintPathRole::SupportMaterial)
        .collect();
    assert!(!support_material_paths.is_empty());
    for path in support_material_paths {
        assert_support_material_metadata(path);
    }
    assert_support_material_bounds(&finalized[0], 0.5, 0.5, 3.5, 3.5);
}

#[test]
fn auto_tree_support_brim_expands_first_layer_support_material_by_orca_lower_bound() {
    let support = support_rect(PrintPathRole::SupportMaterial, 0.0, 0.0, 2.0, 2.0)
        .with_extrusion_role(PrintPathRole::SupportMaterial)
        .with_effective_layer_height_mm(0.16)
        .with_effective_line_width_mm(Some(0.44))
        .with_unsupported_span_mm(Some(1.2))
        .with_seam_gap_mm(0.08);

    let finalized = finalize(
        vec![layer(0, support)],
        json!({
            "support_type": "tree(auto)",
            "tree_support_auto_brim": true,
            "tree_support_brim_width": 1.25,
            "tree_support_wall_count": 1
        }),
        empty_contours(1),
    );

    let support_material = finalized[0]
        .paths()
        .iter()
        .find(|path| path.role() == PrintPathRole::SupportMaterial && path.is_closed())
        .unwrap();
    assert_support_material_metadata(support_material);
    assert!(support_material.is_closed());
    assert_support_material_bounds(&finalized[0], -2.0, -2.0, 4.0, 4.0);
}

#[test]
fn auto_tree_support_brim_ignores_zero_manual_width() {
    let finalized = finalize(
        vec![layer(
            0,
            support_rect(PrintPathRole::SupportMaterial, 0.0, 0.0, 2.0, 2.0),
        )],
        json!({
            "support_type": "tree(auto)",
            "tree_support_auto_brim": true,
            "tree_support_brim_width": 0.0
        }),
        empty_contours(1),
    );

    assert_support_material_bounds(&finalized[0], -2.0, -2.0, 4.0, 4.0);
}

#[test]
fn tree_support_brim_preserves_non_tree_support_paths() {
    let finalized = finalize(
        vec![layer(
            0,
            support_rect(PrintPathRole::SupportMaterial, 0.0, 0.0, 2.0, 2.0),
        )],
        json!({
            "support_type": "normal(auto)",
            "tree_support_auto_brim": true,
            "tree_support_brim_width": 1.25
        }),
        empty_contours(1),
    );

    assert_support_material_bounds(&finalized[0], 0.0, 0.0, 2.0, 2.0);
}

#[test]
fn manual_tree_support_brim_preserves_raft_second_layer_zero_width_and_non_base_paths() {
    for (layer_id, extra) in [
        (
            0,
            json!({
                "support_type": "tree(auto)",
                "tree_support_auto_brim": false,
                "tree_support_brim_width": 1.25,
                "raft_layers": 1,
                "raft_expansion": 0.0,
                "raft_first_layer_expansion": 0.0
            }),
        ),
        (
            1,
            json!({
                "support_type": "tree(auto)",
                "tree_support_auto_brim": false,
                "tree_support_brim_width": 1.25
            }),
        ),
        (
            0,
            json!({
                "support_type": "tree(auto)",
                "tree_support_auto_brim": false,
                "tree_support_brim_width": 0.0
            }),
        ),
    ] {
        let finalized = finalize(
            vec![layer(
                layer_id,
                support_rect(PrintPathRole::SupportMaterial, 0.0, 0.0, 2.0, 2.0),
            )],
            extra,
            empty_contours(layer_id + 1),
        );

        assert_support_material_bounds(layer_by_id(&finalized, layer_id), 0.0, 0.0, 2.0, 2.0);
    }
}

#[test]
fn manual_tree_support_brim_preserves_open_and_non_rectangular_support() {
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

    let finalized = finalize(
        vec![layer_paths(0, vec![open_support.clone(), triangle.clone()])],
        json!({
            "support_type": "tree(auto)",
            "tree_support_auto_brim": false,
            "tree_support_brim_width": 1.25
        }),
        empty_contours(1),
    );

    assert_eq!(finalized[0].paths(), [open_support, triangle]);
}

#[test]
fn manual_tree_support_brim_preserves_interface_support_when_spacing_isolated() {
    let interface = support_rect(PrintPathRole::SupportMaterialInterface, 0.0, 0.0, 2.0, 2.0);

    let finalized = finalize(
        vec![layer(0, interface.clone())],
        json!({
            "support_type": "tree(auto)",
            "tree_support_auto_brim": false,
            "tree_support_brim_width": 1.25,
            "support_ironing": true
        }),
        empty_contours(1),
    );

    assert!(contains_exact_path(&finalized[0], &interface));
    assert_eq!(
        finalized[0]
            .paths()
            .iter()
            .filter(|path| path.role() == PrintPathRole::SupportMaterial)
            .count(),
        0
    );
}
