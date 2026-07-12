use super::*;

fn square_layer_with_id(layer_id: usize, print_z: f64) -> LayerContours {
    LayerContours::new(layer_id, print_z, square_layer().contours().to_vec())
}

fn surface_density_options() -> InfillOptions {
    InfillOptions::new_for_tests(50.0, 0.0, 0.5)
        .with_minimum_sparse_infill_area_for_tests(0.0)
        .with_shell_layers_for_tests(1, 1)
}

#[test]
fn top_surface_density_reduces_only_top_surface_lines() {
    let layers = vec![
        square_layer_with_id(0, 0.2),
        square_layer_with_id(1, 0.4),
        square_layer_with_id(2, 0.6),
    ];
    let options = surface_density_options().with_top_surface_density_for_tests(50.0);

    let infills = generate_infills(&print_layers(&layers), &layers, options).unwrap();

    assert_eq!(infills[0].paths().len(), 4);
    assert_eq!(infills[1].paths().len(), 2);
    assert_eq!(infills[2].paths().len(), 2);
    assert!(
        infills[0]
            .paths()
            .iter()
            .all(|path| path.role() == InfillRole::Solid)
    );
    assert!(
        infills[1]
            .paths()
            .iter()
            .all(|path| path.role() == InfillRole::Sparse)
    );
    assert!(
        infills[2]
            .paths()
            .iter()
            .all(|path| path.role() == InfillRole::Solid)
    );
}

#[test]
fn bottom_surface_density_reduces_only_bottom_surface_lines() {
    let layers = vec![
        square_layer_with_id(0, 0.2),
        square_layer_with_id(1, 0.4),
        square_layer_with_id(2, 0.6),
    ];
    let options = surface_density_options().with_bottom_surface_density_for_tests(50.0);

    let infills = generate_infills(&print_layers(&layers), &layers, options).unwrap();

    assert_eq!(infills[0].paths().len(), 2);
    assert_eq!(infills[1].paths().len(), 2);
    assert_eq!(infills[2].paths().len(), 4);
}

#[test]
fn zero_top_surface_density_emits_no_top_surface_infill_paths() {
    let layers = vec![
        square_layer_with_id(0, 0.2),
        square_layer_with_id(1, 0.4),
        square_layer_with_id(2, 0.6),
    ];
    let options = surface_density_options().with_top_surface_density_for_tests(0.0);

    let infills = generate_infills(&print_layers(&layers), &layers, options).unwrap();

    assert_eq!(infills[0].paths().len(), 4);
    assert_eq!(infills[1].paths().len(), 2);
    assert!(infills[2].paths().is_empty());
}

#[test]
fn bridge_density_override_wins_over_bottom_surface_density() {
    let layers = vec![
        LayerContours::new(0, 0.2, square_layer().contours().to_vec()),
        LayerContours::new(
            1,
            0.4,
            vec![Contour::new(vec![
                Point2::new(10.0, 0.0),
                Point2::new(12.0, 0.0),
                Point2::new(12.0, 2.0),
                Point2::new(10.0, 2.0),
            ])],
        ),
    ];
    let context = InfillBridgeContext::new(
        &layers,
        true,
        crate::bridges::ExtraBridgeLayer::Disabled,
        crate::bridges::CounterboreHoleBridging::None,
    );
    let options = InfillOptions::new_for_tests(100.0, 0.0, 0.5)
        .with_minimum_sparse_infill_area_for_tests(0.0)
        .with_shell_layers_for_tests(2, 0)
        .with_bottom_surface_density_for_tests(50.0)
        .with_bridge_density_for_tests(100.0);

    let infills = generate_infills_with_bridge_context(
        &print_layers(&layers),
        &layers,
        options,
        Some(context),
    )
    .unwrap();

    assert_eq!(infills[1].paths().len(), 4);
}
