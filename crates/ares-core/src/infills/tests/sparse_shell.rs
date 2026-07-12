use super::*;
use crate::SliceOptions;
use serde_json::json;

#[test]
fn sparse_density_generates_solid_shells_and_sparse_interior() {
    let layers = vec![
        LayerContours::new(0, 0.2, square_layer().contours().to_vec()),
        LayerContours::new(1, 0.4, square_layer().contours().to_vec()),
        LayerContours::new(2, 0.6, square_layer().contours().to_vec()),
    ];
    let options = InfillOptions::new_for_tests(50.0, 0.0, 0.5)
        .with_minimum_sparse_infill_area_for_tests(0.0)
        .with_shell_layers_for_tests(1, 1);

    let infills = generate_infills(&print_layers(&layers), &layers, options).unwrap();

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
fn sparse_density_shells_use_solid_spacing_and_no_sparse_anchor_extension() {
    let layers = vec![
        LayerContours::new(0, 0.2, square_layer().contours().to_vec()),
        LayerContours::new(1, 0.4, square_layer().contours().to_vec()),
        LayerContours::new(2, 0.6, square_layer().contours().to_vec()),
    ];
    let options: SliceOptions = serde_json::from_value(json!({
        "sparse_infill_density": 50,
        "sparse_infill_line_width": 0.5,
        "line_width": 0.4,
        "minimum_sparse_infill_area": 0,
        "sparse_infill_pattern": "rectilinear",
        "infill_direction": 0,
        "solid_infill_direction": 0,
        "infill_anchor": 0.25,
        "infill_anchor_max": 20,
        "wall_loops": 0,
        "bottom_shell_layers": 1,
        "bottom_shell_thickness": 0,
        "top_shell_layers": 1,
        "top_shell_thickness": 0
    }))
    .unwrap();
    let options = options.infill_options().unwrap();

    let infills = generate_infills(&print_layers(&layers), &layers, options).unwrap();

    assert_eq!(infills[0].paths().len(), 5);
    assert_eq!(
        infills[0].paths()[0].points(),
        &[Point2::new(0.2, 0.0), Point2::new(0.2, 2.0)]
    );
    assert_eq!(infills[1].paths().len(), 2);
    assert_eq!(
        infills[1].paths()[0].points(),
        &[Point2::new(2.25, 0.5), Point2::new(-0.25, 0.5)]
    );
    assert_eq!(infills[2].paths().len(), 5);
    assert_eq!(
        infills[2].paths()[0].points(),
        &[Point2::new(0.2, 0.0), Point2::new(0.2, 2.0)]
    );
}
