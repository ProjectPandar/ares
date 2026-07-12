use super::*;

#[test]
fn alternate_extra_wall_adds_one_wall_on_odd_layers_with_sparse_infill() {
    let layers = [
        LayerContours::new(0, 0.2, vec![rectangle(0.0, 0.0, 5.0, 5.0)]),
        LayerContours::new(1, 0.4, vec![rectangle(0.0, 0.0, 5.0, 5.0)]),
        LayerContours::new(2, 0.6, vec![rectangle(0.0, 0.0, 5.0, 5.0)]),
    ];
    let options = PerimeterOptions::new(
        2,
        0.4,
        0.4,
        WallDirection::CounterClockwise,
        WallSequence::InnerOuter,
    )
    .with_alternate_extra_wall(true)
    .with_sparse_infill_density_percent(20.0);

    let perimeters = generate_perimeters(&layers, options).unwrap();

    assert_eq!(perimeters[0].paths().len(), 2);
    assert_eq!(perimeters[1].paths().len(), 3);
    assert_eq!(perimeters[2].paths().len(), 2);
    assert_eq!(
        perimeters[1].paths()[0].points(),
        rectangle_points(
            0.7570796326794897,
            0.7570796326794897,
            4.24292036732051,
            4.24292036732051
        )
    );
    assert_eq!(
        perimeters[1].paths()[1].points(),
        rectangle_points(0.4, 0.4, 4.6, 4.6)
    );
    assert_eq!(perimeters[1].paths()[2].role(), PerimeterRole::External);
}

#[test]
fn alternate_extra_wall_is_inert_without_sparse_infill() {
    let layers = [
        LayerContours::new(0, 0.2, vec![rectangle(0.0, 0.0, 5.0, 5.0)]),
        LayerContours::new(1, 0.4, vec![rectangle(0.0, 0.0, 5.0, 5.0)]),
    ];
    let options = PerimeterOptions::new(
        2,
        0.4,
        0.4,
        WallDirection::CounterClockwise,
        WallSequence::InnerOuter,
    )
    .with_alternate_extra_wall(true)
    .with_sparse_infill_density_percent(0.0);

    let perimeters = generate_perimeters(&layers, options).unwrap();

    assert_eq!(perimeters[0].paths().len(), 2);
    assert_eq!(perimeters[1].paths().len(), 2);
}
