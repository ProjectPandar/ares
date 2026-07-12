use super::*;

#[test]
fn only_one_wall_first_layer_limits_first_layer_to_external_wall() {
    let layers = [
        LayerContours::new(0, 0.2, vec![rectangle(0.0, 0.0, 4.0, 4.0)]),
        LayerContours::new(1, 0.4, vec![rectangle(0.0, 0.0, 4.0, 4.0)]),
    ];

    let options = PerimeterOptions::new(
        3,
        0.4,
        0.4,
        WallDirection::CounterClockwise,
        WallSequence::InnerOuter,
    )
    .with_only_one_wall_first_layer(true);
    let perimeters = generate_perimeters(&layers, options).unwrap();

    assert_eq!(perimeters[0].paths().len(), 1);
    assert_eq!(perimeters[0].paths()[0].role(), PerimeterRole::External);
    assert_eq!(perimeters[1].paths().len(), 3);
    assert_eq!(perimeters[1].paths()[0].role(), PerimeterRole::Internal);
    assert_eq!(perimeters[1].paths()[1].role(), PerimeterRole::Internal);
    assert_eq!(perimeters[1].paths()[2].role(), PerimeterRole::External);
}
