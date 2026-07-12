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

fn rectangle_layer_with_id(
    layer_id: usize,
    print_z: f64,
    bounds: (f64, f64, f64, f64),
) -> LayerContours {
    let (min_x, min_y, max_x, max_y) = bounds;
    LayerContours::new(
        layer_id,
        print_z,
        vec![Contour::new(vec![
            Point2::new(min_x, min_y),
            Point2::new(max_x, min_y),
            Point2::new(max_x, max_y),
            Point2::new(min_x, max_y),
        ])],
    )
}

fn bridge_angle_options(bridge_angle_degrees: f64) -> InfillOptions {
    InfillOptions::new_for_tests(100.0, 0.0, 0.5)
        .with_minimum_sparse_infill_area_for_tests(0.0)
        .with_bottom_surface_pattern_for_tests(InfillPattern::AlignedRectilinear)
        .with_shell_layers_for_tests(2, 0)
        .with_bridge_angle_for_tests(bridge_angle_degrees)
}

fn second_layer_points(infills: &[LayerInfills]) -> Vec<&[Point2]> {
    infills[1].paths().iter().map(InfillPath::points).collect()
}

#[test]
fn bridge_angle_without_bridge_context_preserves_bottom_surface_direction() {
    let layers = vec![
        square_layer_with_id(0, 0.2, 0.0),
        square_layer_with_id(1, 0.4, 10.0),
    ];

    let infills =
        generate_infills(&print_layers(&layers), &layers, bridge_angle_options(90.0)).unwrap();

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
fn bridge_angle_with_support_enabled_preserves_bottom_surface_direction() {
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
        bridge_angle_options(90.0),
        Some(context),
    )
    .unwrap();

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
fn bridge_angle_overrides_fully_unsupported_bottom_surface_direction() {
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
        bridge_angle_options(90.0),
        Some(context),
    )
    .unwrap();

    assert_eq!(
        second_layer_points(&infills),
        vec![
            &[Point2::new(12.0, 0.25), Point2::new(10.0, 0.25)][..],
            &[Point2::new(12.0, 0.75), Point2::new(10.0, 0.75)][..],
            &[Point2::new(12.0, 1.25), Point2::new(10.0, 1.25)][..],
            &[Point2::new(12.0, 1.75), Point2::new(10.0, 1.75)][..],
        ]
    );
}

#[test]
fn zero_bridge_angle_auto_detects_fully_unsupported_bottom_surface_direction() {
    let layers = vec![
        rectangle_layer_with_id(0, 0.2, (0.0, 0.0, 4.0, 2.0)),
        rectangle_layer_with_id(1, 0.4, (10.0, 0.0, 14.0, 2.0)),
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
        bridge_angle_options(0.0),
        Some(context),
    )
    .unwrap();

    assert_eq!(
        second_layer_points(&infills),
        vec![
            &[Point2::new(14.0, 0.25), Point2::new(10.0, 0.25)][..],
            &[Point2::new(14.0, 0.75), Point2::new(10.0, 0.75)][..],
            &[Point2::new(14.0, 1.25), Point2::new(10.0, 1.25)][..],
            &[Point2::new(14.0, 1.75), Point2::new(10.0, 1.75)][..],
        ]
    );
}

#[test]
fn zero_bridge_angle_preserves_current_direction_for_square_bounds() {
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
        bridge_angle_options(0.0),
        Some(context),
    )
    .unwrap();

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
