use super::*;
use crate::{LayerPerimeters, PerimeterPath, PerimeterRole, Point2};

#[test]
fn maps_overhang_perimeter_to_overhang_print_path_role() {
    let perimeters = vec![LayerPerimeters::new(
        1,
        0.4,
        vec![
            PerimeterPath::new(
                PerimeterRole::Overhang,
                vec![
                    Point2::new(0.0, 0.0),
                    Point2::new(1.0, 0.0),
                    Point2::new(1.0, 1.0),
                ],
            )
            .unwrap(),
        ],
    )];

    let output = generate_print_paths(
        PrintPathInput::new(
            &sample_skirts(1, 0.4),
            &sample_brims(1, 0.4),
            &perimeters,
            &sample_gap_fills(1, 0.4),
            &sample_infills(1, 0.4),
        ),
        ShellLayerOptions::new(1, 1),
        false,
        false,
    )
    .unwrap();

    assert_eq!(
        output[0].paths()[0].role(),
        PrintPathRole::OverhangPerimeter
    );
    assert_eq!(
        PrintPathRole::OverhangPerimeter.as_str(),
        "overhang_perimeter"
    );
}
