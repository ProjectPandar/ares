use super::*;

#[test]
fn elephant_foot_layers_density_ramps_internal_solid_layers() {
    let layers = vec![
        LayerContours::new(0, 0.2, square_layer().contours().to_vec()),
        LayerContours::new(1, 0.4, square_layer().contours().to_vec()),
        LayerContours::new(2, 0.6, square_layer().contours().to_vec()),
        LayerContours::new(3, 0.8, square_layer().contours().to_vec()),
        LayerContours::new(4, 1.0, square_layer().contours().to_vec()),
    ];
    let options = InfillOptions::new_for_tests(100.0, 0.0, 0.5)
        .with_minimum_sparse_infill_area_for_tests(0.0)
        .with_shell_layers_for_tests(1, 1)
        .with_elephant_foot_layers_density_for_tests(50.0)
        .with_elephant_foot_compensation_layers_for_tests(2);

    let infills = generate_infills(&print_layers(&layers), &layers, options).unwrap();

    assert_eq!(infills[0].paths().len(), 4);
    assert_eq!(infills[1].paths().len(), 2);
    assert_eq!(infills[2].paths().len(), 3);
    assert_eq!(infills[3].paths().len(), 4);
    assert_eq!(infills[4].paths().len(), 4);
}

#[test]
fn elephant_foot_layers_density_ramp_stops_after_configured_layers() {
    let layers = vec![
        LayerContours::new(0, 0.2, square_layer().contours().to_vec()),
        LayerContours::new(1, 0.4, square_layer().contours().to_vec()),
        LayerContours::new(2, 0.6, square_layer().contours().to_vec()),
        LayerContours::new(3, 0.8, square_layer().contours().to_vec()),
        LayerContours::new(4, 1.0, square_layer().contours().to_vec()),
    ];
    let options = InfillOptions::new_for_tests(100.0, 0.0, 0.5)
        .with_minimum_sparse_infill_area_for_tests(0.0)
        .with_shell_layers_for_tests(1, 1)
        .with_elephant_foot_layers_density_for_tests(50.0)
        .with_elephant_foot_compensation_layers_for_tests(1);

    let infills = generate_infills(&print_layers(&layers), &layers, options).unwrap();

    assert_eq!(infills[1].paths().len(), 2);
    assert_eq!(infills[2].paths().len(), 4);
}
