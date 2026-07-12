use super::*;

fn square_layer_with_id(layer_id: usize, print_z: f64, x_offset: f64) -> LayerContours {
    LayerContours::new(
        layer_id,
        print_z,
        vec![Contour::new(vec![
            Point2::new(x_offset, 0.0),
            Point2::new(x_offset + 2.0, 0.0),
            Point2::new(x_offset + 2.0, 2.0),
            Point2::new(x_offset, 2.0),
        ])],
    )
}

fn bridge_density_options(bridge_density_percent: f64) -> InfillOptions {
    InfillOptions::new_for_tests(100.0, 0.0, 0.5)
        .with_minimum_sparse_infill_area_for_tests(0.0)
        .with_bottom_surface_pattern_for_tests(InfillPattern::AlignedRectilinear)
        .with_shell_layers_for_tests(2, 0)
        .with_bridge_density_for_tests(bridge_density_percent)
}

fn second_layer_points(infills: &[LayerInfills]) -> Vec<&[Point2]> {
    infills[1].paths().iter().map(InfillPath::points).collect()
}

#[test]
fn bridge_density_without_bridge_context_preserves_bottom_surface_spacing() {
    let layers = vec![
        square_layer_with_id(0, 0.2, 0.0),
        square_layer_with_id(1, 0.4, 10.0),
    ];

    let infills = generate_infills(
        &print_layers(&layers),
        &layers,
        bridge_density_options(50.0),
    )
    .unwrap();

    assert_eq!(infills[1].paths().len(), 4);
    assert_eq!(
        second_layer_points(&infills),
        vec![
            &[Point2::new(10.25, 0.0), Point2::new(10.25, 2.0)][..],
            &[Point2::new(10.75, 0.0), Point2::new(10.75, 2.0)][..],
            &[Point2::new(11.25, 0.0), Point2::new(11.25, 2.0)][..],
            &[Point2::new(11.75, 0.0), Point2::new(11.75, 2.0)][..],
        ]
    );
}

#[test]
fn bridge_density_with_support_enabled_preserves_bottom_surface_spacing() {
    let layers = vec![
        square_layer_with_id(0, 0.2, 0.0),
        square_layer_with_id(1, 0.4, 10.0),
    ];
    let context = InfillBridgeContext::new(
        &layers,
        false,
        crate::bridges::ExtraBridgeLayer::Disabled,
        crate::bridges::CounterboreHoleBridging::None,
    );

    let infills = generate_infills_with_bridge_context(
        &print_layers(&layers),
        &layers,
        bridge_density_options(50.0),
        Some(context),
    )
    .unwrap();

    assert_eq!(infills[1].paths().len(), 4);
}

#[test]
fn lower_bridge_density_reduces_fully_unsupported_bottom_surface_lines() {
    let layers = vec![
        square_layer_with_id(0, 0.2, 0.0),
        square_layer_with_id(1, 0.4, 10.0),
    ];
    let context = InfillBridgeContext::new(
        &layers,
        true,
        crate::bridges::ExtraBridgeLayer::Disabled,
        crate::bridges::CounterboreHoleBridging::None,
    );

    let infills = generate_infills_with_bridge_context(
        &print_layers(&layers),
        &layers,
        bridge_density_options(50.0),
        Some(context),
    )
    .unwrap();

    assert_eq!(
        second_layer_points(&infills),
        vec![
            &[Point2::new(10.5, 0.0), Point2::new(10.5, 2.0)][..],
            &[Point2::new(11.5, 0.0), Point2::new(11.5, 2.0)][..],
        ]
    );
}

#[test]
fn higher_bridge_density_adds_fully_unsupported_bottom_surface_lines() {
    let layers = vec![
        square_layer_with_id(0, 0.2, 0.0),
        square_layer_with_id(1, 0.4, 10.0),
    ];
    let context = InfillBridgeContext::new(
        &layers,
        true,
        crate::bridges::ExtraBridgeLayer::Disabled,
        crate::bridges::CounterboreHoleBridging::None,
    );

    let infills = generate_infills_with_bridge_context(
        &print_layers(&layers),
        &layers,
        bridge_density_options(120.0),
        Some(context),
    )
    .unwrap();

    assert_eq!(infills[1].paths().len(), 5);
}

#[test]
fn default_bridge_density_preserves_fully_unsupported_bottom_surface_spacing() {
    let layers = vec![
        square_layer_with_id(0, 0.2, 0.0),
        square_layer_with_id(1, 0.4, 10.0),
    ];
    let context = InfillBridgeContext::new(
        &layers,
        true,
        crate::bridges::ExtraBridgeLayer::Disabled,
        crate::bridges::CounterboreHoleBridging::None,
    );

    let infills = generate_infills_with_bridge_context(
        &print_layers(&layers),
        &layers,
        bridge_density_options(100.0),
        Some(context),
    )
    .unwrap();

    assert_eq!(infills[1].paths().len(), 4);
}

#[test]
fn bridge_density_composes_with_bridge_angle_on_fully_unsupported_bottom_surface() {
    let layers = vec![
        square_layer_with_id(0, 0.2, 0.0),
        square_layer_with_id(1, 0.4, 10.0),
    ];
    let context = InfillBridgeContext::new(
        &layers,
        true,
        crate::bridges::ExtraBridgeLayer::Disabled,
        crate::bridges::CounterboreHoleBridging::None,
    );
    let options = bridge_density_options(50.0).with_bridge_angle_for_tests(90.0);

    let infills = generate_infills_with_bridge_context(
        &print_layers(&layers),
        &layers,
        options,
        Some(context),
    )
    .unwrap();

    assert_eq!(
        second_layer_points(&infills),
        vec![
            &[Point2::new(12.0, 0.5), Point2::new(10.0, 0.5)][..],
            &[Point2::new(12.0, 1.5), Point2::new(10.0, 1.5)][..],
        ]
    );
}
