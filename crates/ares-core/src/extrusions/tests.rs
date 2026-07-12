use super::*;
use crate::{Layer, Point2, ToolpathMove, ToolpathMoveKind};

mod adaptive_volumetric;
mod metadata;
mod overhang;
mod role_filament_extrusion;
mod small_area;
mod solid_infill;
mod support_filament_extrusion;
mod thick_bridges;
mod top_bottom_solid_surface;

#[test]
fn generates_absolute_e_for_print_moves_and_none_for_travel() {
    let layers = [Layer::new(0, 0.2, 0.2)];
    let moves = [LayerToolpathMoves::new(
        0,
        0.2,
        vec![
            ToolpathMove::new(
                ToolpathMoveKind::Travel,
                PrintPathRole::SparseInfill,
                Point2::new(0.0, 0.0),
            ),
            ToolpathMove::new(
                ToolpathMoveKind::Print,
                PrintPathRole::SparseInfill,
                Point2::new(1.0, 0.0),
            ),
            ToolpathMove::new(
                ToolpathMoveKind::Print,
                PrintPathRole::SparseInfill,
                Point2::new(1.0, 1.0),
            ),
        ],
    )];
    let options = ExtrusionOptions::new_for_tests(0.4, 2.0, 0.2, (0.2, 0.0), 0.2);

    let output = generate_extrusion_moves(&layers, &moves, options).unwrap();

    assert_eq!(output[0].moves()[0].e_position(), None);
    assert_eq!(output[0].moves()[1].e_position(), Some(0.01));
    assert_eq!(output[0].moves()[2].e_position(), Some(0.02));
    assert_eq!(output[0].total_extrusion_mm(), 0.02);
}

#[test]
fn skirt_width_uses_line_width_fallback() {
    let options = ExtrusionOptions::new_for_tests(0.4, 2.0, 0.3, (0.2, 0.0), 0.2);
    assert_eq!(options.width_for_role(PrintPathRole::Skirt), 0.3);

    let automatic = ExtrusionOptions::new_for_tests(0.4, 2.0, 0.0, (0.2, 0.0), 0.2);
    assert_eq!(automatic.width_for_role(PrintPathRole::Skirt), 0.45);
}

#[test]
fn brim_width_uses_line_width_fallback() {
    let options = ExtrusionOptions::new_for_tests(0.4, 2.0, 0.3, (0.2, 0.0), 0.2);
    assert_eq!(options.width_for_role(PrintPathRole::Brim), 0.3);

    let automatic = ExtrusionOptions::new_for_tests(0.4, 2.0, 0.0, (0.2, 0.0), 0.2);
    assert_eq!(automatic.width_for_role(PrintPathRole::Brim), 0.45);
}

#[test]
fn internal_perimeter_width_uses_line_width_fallback() {
    let options = ExtrusionOptions::new_for_tests(0.4, 2.0, 0.5, (0.3, 0.0), 0.2);

    assert_eq!(
        options.width_for_role(PrintPathRole::InternalPerimeter),
        0.5
    );
}

#[test]
fn internal_perimeter_width_uses_inner_wall_line_width() {
    let options = ExtrusionOptions::new_for_tests(0.4, 2.0, 0.5, (0.3, 0.25), 0.2);

    assert_eq!(
        options.width_for_role(PrintPathRole::InternalPerimeter),
        0.25
    );
}

#[test]
fn bridge_flow_scales_extrusion_per_mm() {
    assert_eq!(PrintPathRole::Bridge.as_str(), "bridge");
    let base = ExtrusionOptions::new_for_tests(0.4, 2.0, 0.4, (0.0, 0.0), 0.0);
    let bridge = base.with_bridge_flow(0.5);
    let base_e = base
        .extrusion_per_mm(PrintPathRole::ExternalPerimeter, 0.2)
        .unwrap();
    let bridge_e = bridge.extrusion_per_mm(PrintPathRole::Bridge, 0.2).unwrap();
    assert_eq!(
        (bridge_e * 1_000_000.0).round(),
        (base_e * 0.5 * 1_000_000.0).round()
    );
}

#[test]
fn brim_flow_scales_only_brim_extrusion_per_mm() {
    assert_eq!(PrintPathRole::Brim.as_str(), "brim");
    let base = ExtrusionOptions::new_for_tests(0.4, 2.0, 0.4, (0.0, 0.0), 0.0);
    let scaled = base.with_bridge_flow(0.5).with_brim_flow_ratio(0.25);
    let base_e = base
        .extrusion_per_mm(PrintPathRole::ExternalPerimeter, 0.2)
        .unwrap();
    let brim_e = scaled.extrusion_per_mm(PrintPathRole::Brim, 0.2).unwrap();
    let bridge_e = scaled.extrusion_per_mm(PrintPathRole::Bridge, 0.2).unwrap();
    let skirt_e = scaled.extrusion_per_mm(PrintPathRole::Skirt, 0.2).unwrap();
    let infill_e = scaled
        .extrusion_per_mm(PrintPathRole::SparseInfill, 0.2)
        .unwrap();
    let perimeter_e = scaled
        .extrusion_per_mm(PrintPathRole::ExternalPerimeter, 0.2)
        .unwrap();

    assert_eq!(
        (brim_e * 1_000_000.0).round(),
        (base_e * 0.25 * 1_000_000.0).round()
    );
    assert_eq!(
        (bridge_e * 1_000_000.0).round(),
        (base_e * 0.5 * 1_000_000.0).round()
    );
    assert_eq!(
        (skirt_e * 1_000_000.0).round(),
        (base_e * 1_000_000.0).round()
    );
    assert_eq!(
        (infill_e * 1_000_000.0).round(),
        (base_e * 1_000_000.0).round()
    );
    assert_eq!(
        (perimeter_e * 1_000_000.0).round(),
        (base_e * 1_000_000.0).round()
    );
}

#[test]
fn internal_bridge_flow_scales_only_internal_bridge_extrusion_per_mm() {
    assert_eq!(PrintPathRole::InternalBridge.as_str(), "internal_bridge");
    let base = ExtrusionOptions::new_for_tests(0.4, 2.0, 0.4, (0.0, 0.0), 0.0);
    let scaled = base.with_bridge_flow(0.5).with_internal_bridge_flow(0.25);
    let base_e = base
        .extrusion_per_mm(PrintPathRole::ExternalPerimeter, 0.2)
        .unwrap();
    let bridge_e = scaled.extrusion_per_mm(PrintPathRole::Bridge, 0.2).unwrap();
    let internal_bridge_e = scaled
        .extrusion_per_mm(PrintPathRole::InternalBridge, 0.2)
        .unwrap();
    let infill_e = scaled
        .extrusion_per_mm(PrintPathRole::SparseInfill, 0.2)
        .unwrap();

    assert_eq!(
        (bridge_e * 1_000_000.0).round(),
        (base_e * 0.5 * 1_000_000.0).round()
    );
    assert_eq!(
        (internal_bridge_e * 1_000_000.0).round(),
        (base_e * 0.25 * 1_000_000.0).round()
    );
    assert_eq!(
        (infill_e * 1_000_000.0).round(),
        (base_e * 1_000_000.0).round()
    );
}

#[test]
fn first_layer_flow_ratio_scales_only_supported_first_layer_roles() {
    let base = ExtrusionOptions::new_for_tests(0.4, 2.0, 0.4, (0.0, 0.0), 0.0);
    let scaled = base.with_first_layer_flow_ratio(0.25);
    let base_e = base
        .extrusion_per_mm(PrintPathRole::ExternalPerimeter, 0.2)
        .unwrap();

    for role in [
        PrintPathRole::ExternalPerimeter,
        PrintPathRole::InternalPerimeter,
        PrintPathRole::SparseInfill,
    ] {
        let first_layer_e = scaled.extrusion_per_mm_for_layer(role, 0.2, true).unwrap();
        assert_eq!(
            (first_layer_e * 1_000_000.0).round(),
            (base_e * 0.25 * 1_000_000.0).round()
        );
    }

    for role in [
        PrintPathRole::Skirt,
        PrintPathRole::Brim,
        PrintPathRole::Bridge,
        PrintPathRole::InternalBridge,
    ] {
        let first_layer_e = scaled.extrusion_per_mm_for_layer(role, 0.2, true).unwrap();
        assert_eq!(
            (first_layer_e * 1_000_000.0).round(),
            (base.extrusion_per_mm(role, 0.2).unwrap() * 1_000_000.0).round()
        );
    }
}

#[test]
fn print_flow_ratio_scales_all_roles_and_composes_with_role_ratios() {
    let base = ExtrusionOptions::new_for_tests(0.4, 2.0, 0.4, (0.0, 0.0), 0.0);
    let scaled = base.with_print_flow_ratio(1.5).with_brim_flow_ratio(0.5);
    let base_e = base
        .extrusion_per_mm(PrintPathRole::ExternalPerimeter, 0.2)
        .unwrap();

    let external_e = scaled
        .extrusion_per_mm(PrintPathRole::ExternalPerimeter, 0.2)
        .unwrap();
    let brim_e = scaled.extrusion_per_mm(PrintPathRole::Brim, 0.2).unwrap();
    let bridge_e = scaled.extrusion_per_mm(PrintPathRole::Bridge, 0.2).unwrap();

    assert_eq!(
        (external_e * 1_000_000.0).round(),
        (base_e * 1.5 * 1_000_000.0).round()
    );
    assert_eq!(
        (brim_e * 1_000_000.0).round(),
        (base_e * 1.5 * 0.5 * 1_000_000.0).round()
    );
    assert_eq!(
        (bridge_e * 1_000_000.0).round(),
        (base_e * 1.5 * 1_000_000.0).round()
    );
}

#[test]
fn generated_extrusion_moves_apply_first_layer_flow_only_on_layer_zero() {
    let layers = [Layer::new(0, 0.2, 0.2), Layer::new(1, 0.2, 0.4)];
    let moves = [
        LayerToolpathMoves::new(
            0,
            0.2,
            vec![
                ToolpathMove::new(
                    ToolpathMoveKind::Travel,
                    PrintPathRole::ExternalPerimeter,
                    Point2::new(0.0, 0.0),
                ),
                ToolpathMove::new(
                    ToolpathMoveKind::Print,
                    PrintPathRole::ExternalPerimeter,
                    Point2::new(1.0, 0.0),
                ),
            ],
        ),
        LayerToolpathMoves::new(
            1,
            0.4,
            vec![
                ToolpathMove::new(
                    ToolpathMoveKind::Travel,
                    PrintPathRole::ExternalPerimeter,
                    Point2::new(0.0, 0.0),
                ),
                ToolpathMove::new(
                    ToolpathMoveKind::Print,
                    PrintPathRole::ExternalPerimeter,
                    Point2::new(1.0, 0.0),
                ),
            ],
        ),
    ];
    let options = ExtrusionOptions::new_for_tests(0.4, 2.0, 0.4, (0.0, 0.0), 0.0)
        .with_first_layer_flow_ratio(0.5);

    let output = generate_extrusion_moves(&layers, &moves, options).unwrap();

    assert_eq!(output[0].moves()[1].e_position(), Some(0.011366));
    assert_eq!(output[1].moves()[1].e_position(), Some(0.034098));
    assert_eq!(output[0].total_extrusion_mm(), 0.011366);
    assert_eq!(output[1].total_extrusion_mm(), 0.022732);
}

#[test]
fn generated_extrusion_moves_use_effective_sparse_infill_height() {
    let layers = [Layer::new(1, 0.2, 0.4)];
    let base_moves = [LayerToolpathMoves::new(
        1,
        0.4,
        vec![
            ToolpathMove::new(
                ToolpathMoveKind::Travel,
                PrintPathRole::SparseInfill,
                Point2::new(0.0, 0.0),
            ),
            ToolpathMove::new(
                ToolpathMoveKind::Print,
                PrintPathRole::SparseInfill,
                Point2::new(1.0, 0.0),
            ),
        ],
    )];
    let combined_moves = [LayerToolpathMoves::new(
        1,
        0.4,
        vec![
            ToolpathMove::new(
                ToolpathMoveKind::Travel,
                PrintPathRole::SparseInfill,
                Point2::new(0.0, 0.0),
            )
            .with_effective_layer_height_mm(Some(0.4)),
            ToolpathMove::new(
                ToolpathMoveKind::Print,
                PrintPathRole::SparseInfill,
                Point2::new(1.0, 0.0),
            )
            .with_effective_layer_height_mm(Some(0.4)),
        ],
    )];
    let options = ExtrusionOptions::new_for_tests(0.4, 2.0, 0.5, (0.0, 0.0), 0.5);

    let base = generate_extrusion_moves(&layers, &base_moves, options.clone()).unwrap();
    let combined = generate_extrusion_moves(&layers, &combined_moves, options).unwrap();

    assert!(combined[0].total_extrusion_mm() > base[0].total_extrusion_mm());
}

#[test]
fn initial_layer_line_width_changes_only_supported_first_layer_roles() {
    let base = ExtrusionOptions::new_for_tests(0.4, 2.0, 0.4, (0.0, 0.0), 0.0);
    let wide = base.with_initial_layer_line_width(0.6);

    for role in [
        PrintPathRole::Skirt,
        PrintPathRole::Brim,
        PrintPathRole::ExternalPerimeter,
        PrintPathRole::InternalPerimeter,
        PrintPathRole::SparseInfill,
    ] {
        let normal = wide.extrusion_per_mm_for_layer(role, 0.2, false).unwrap();
        let first = wide.extrusion_per_mm_for_layer(role, 0.2, true).unwrap();

        assert_eq!(
            (normal * 1_000_000.0).round(),
            (base.extrusion_per_mm(role, 0.2).unwrap() * 1_000_000.0).round()
        );
        assert!(first > normal);
    }
}

#[test]
fn initial_layer_line_width_composes_with_first_layer_flow_ratio() {
    let base = ExtrusionOptions::new_for_tests(0.4, 2.0, 0.4, (0.0, 0.0), 0.0)
        .with_initial_layer_line_width(0.6);
    let scaled = base.with_first_layer_flow_ratio(0.5);
    let first = base
        .extrusion_per_mm_for_layer(PrintPathRole::ExternalPerimeter, 0.2, true)
        .unwrap();
    let scaled_first = scaled
        .extrusion_per_mm_for_layer(PrintPathRole::ExternalPerimeter, 0.2, true)
        .unwrap();

    assert_eq!(
        (scaled_first * 1_000_000.0).round(),
        (first * 0.5 * 1_000_000.0).round()
    );
}

#[test]
fn initial_layer_line_width_does_not_change_bridge_roles() {
    let base = ExtrusionOptions::new_for_tests(0.4, 2.0, 0.4, (0.0, 0.0), 0.0)
        .with_bridge_flow(0.5)
        .with_internal_bridge_flow(0.25);
    let wide = base.with_initial_layer_line_width(0.6);

    for role in [PrintPathRole::Bridge, PrintPathRole::InternalBridge] {
        assert_eq!(
            (wide.extrusion_per_mm_for_layer(role, 0.2, true).unwrap() * 1_000_000.0).round(),
            (base.extrusion_per_mm_for_layer(role, 0.2, true).unwrap() * 1_000_000.0).round()
        );
    }
}

#[test]
fn rejects_mismatched_layer_and_move_inputs() {
    let layers = [Layer::new(0, 0.2, 0.2)];
    let moves = [LayerToolpathMoves::new(1, 0.2, Vec::new())];
    let options = ExtrusionOptions::new_for_tests(0.4, 2.0, 0.2, (0.2, 0.0), 0.2);

    assert!(matches!(
        generate_extrusion_moves(&layers, &moves, options),
        Err(SliceError::InvalidInput(_))
    ));
}
