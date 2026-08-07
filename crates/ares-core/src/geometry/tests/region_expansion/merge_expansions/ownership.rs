use super::*;

#[test]
fn task22o33_empty_expansions_preserve_topology_order_and_point_buffers() {
    let src = vec![
        expolygon(
            &[(0, 0), (100, 0), (100, 100), (0, 100)],
            vec![polygon(&[(20, 20), (20, 80), (80, 80), (80, 20)])],
        ),
        square(200, 300),
        square(400, 500),
    ];
    let expected = vec![
        (
            vec![(0, 0), (100, 0), (100, 100), (0, 100)],
            vec![vec![(20, 20), (20, 80), (80, 80), (80, 20)]],
        ),
        (vec![(200, 0), (300, 0), (300, 100), (200, 100)], vec![]),
        (vec![(400, 0), (500, 0), (500, 100), (400, 100)], vec![]),
    ];
    let pointers = src
        .iter()
        .map(|expolygon| {
            (
                expolygon.contour().points().as_ptr(),
                expolygon
                    .holes()
                    .iter()
                    .map(|hole| hole.points().as_ptr())
                    .collect::<Vec<_>>(),
            )
        })
        .collect::<Vec<_>>();

    let actual = MERGE(src, Vec::new(), CoordinateScale::Normal).unwrap();
    assert_eq!(snapshot(&actual), expected);
    for (expolygon, (contour, holes)) in actual.iter().zip(pointers) {
        assert_eq!(expolygon.contour().points().as_ptr(), contour);
        assert_eq!(
            expolygon
                .holes()
                .iter()
                .map(|hole| hole.points().as_ptr())
                .collect::<Vec<_>>(),
            holes
        );
    }
}

#[test]
fn task22o33_moves_leading_interior_and_trailing_sources_without_cloning() {
    let src = vec![
        square(0, 100),
        square(200, 300),
        square(400, 500),
        square(600, 700),
    ];
    let untouched = [0, 2, 3].map(|index| src[index].contour().points().as_ptr());
    let source_pointer = src[1].contour().points().as_ptr();
    let expanded = vec![expansion(
        1,
        99,
        &[(300, 20), (360, 20), (360, 80), (300, 80)],
    )];
    let expansion_pointer = expanded[0].polygon.points().as_ptr();

    let actual = MERGE(src, expanded, CoordinateScale::LargeBed).unwrap();
    assert_eq!(actual.len(), 4);
    for (output, pointer) in [0, 2, 3].into_iter().zip(untouched) {
        assert_eq!(actual[output].contour().points().as_ptr(), pointer);
    }
    let merged_pointer = actual[1].contour().points().as_ptr();
    assert_ne!(merged_pointer, source_pointer);
    assert_ne!(merged_pointer, expansion_pointer);
}
