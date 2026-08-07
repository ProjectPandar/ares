use super::*;

#[test]
fn task22o33_groups_unsorted_records_and_matches_complete_oracle_order() {
    let src = vec![
        square(0, 100),
        square(300, 400),
        square(600, 700),
        square(900, 1000),
    ];
    let expanded = vec![
        expansion(2, 91, &[(700, 20), (760, 20), (760, 80), (700, 80)]),
        expansion(0, 7, &[(100, 20), (160, 20), (160, 80), (100, 80)]),
        expansion(2, 3, &[(650, -40), (690, -40), (690, 0), (650, 0)]),
    ];

    let actual = MERGE(src, expanded, CoordinateScale::Normal).unwrap();
    assert_eq!(
        snapshot(&actual),
        vec![
            (
                vec![
                    (110, 10),
                    (170, 10),
                    (170, 90),
                    (110, 90),
                    (110, 110),
                    (-10, 110),
                    (-10, -10),
                    (110, -10),
                ],
                vec![],
            ),
            (vec![(300, 0), (400, 0), (400, 100), (300, 100)], vec![]),
            (
                vec![
                    (700, -10),
                    (710, -10),
                    (710, 10),
                    (770, 10),
                    (770, 90),
                    (710, 90),
                    (710, 110),
                    (590, 110),
                    (590, -10),
                    (640, -10),
                    (640, -50),
                    (700, -50),
                ],
                vec![],
            ),
            (vec![(900, 0), (1000, 0), (1000, 100), (900, 100)], vec![]),
        ]
    );
}

#[test]
fn task22o33_appends_source_contour_then_hole_and_matches_safety_offset_oracle() {
    let source = expolygon(
        &[(0, 0), (200, 0), (200, 200), (0, 200)],
        vec![polygon(&[(50, 50), (50, 150), (150, 150), (150, 50)])],
    );
    let expanded = vec![expansion(
        0,
        4,
        &[(200, 80), (260, 80), (260, 120), (200, 120)],
    )];

    let actual = MERGE(vec![source], expanded, CoordinateScale::Normal).unwrap();
    assert_eq!(
        snapshot(&actual),
        vec![(
            vec![
                (210, 70),
                (270, 70),
                (270, 130),
                (210, 130),
                (210, 210),
                (-10, 210),
                (-10, -10),
                (210, -10),
            ],
            vec![vec![(60, 60), (60, 140), (140, 140), (140, 60)]],
        )]
    );
}

#[test]
fn task22o33_disconnected_merge_keeps_source_component_for_both_scales() {
    let expanded = || {
        vec![expansion(
            0,
            12,
            &[(300, 0), (400, 0), (400, 100), (300, 100)],
        )]
    };
    let expected = vec![(vec![(110, 110), (-10, 110), (-10, -10), (110, -10)], vec![])];

    for scale in [CoordinateScale::Normal, CoordinateScale::LargeBed] {
        let actual = MERGE(vec![square(0, 100)], expanded(), scale).unwrap();
        assert_eq!(snapshot(&actual), expected);
    }
}

#[test]
fn task22o33_safety_offset_uses_the_source_miter_limit() {
    let triangle = [(0, 0), (100, 0), (50, 53)];
    let actual = MERGE(
        vec![expolygon(&triangle, Vec::new())],
        vec![expansion(0, 0, &triangle)],
        CoordinateScale::Normal,
    )
    .unwrap();

    assert_eq!(
        snapshot(&actual),
        vec![(vec![(50, 68), (-23, -10), (123, -10)], vec![])]
    );
}
