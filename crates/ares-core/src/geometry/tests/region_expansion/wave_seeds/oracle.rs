use super::*;
use crate::geometry::WaveSeed;
use crate::geometry::region_expansion::{expanded_source_paths_for_test, sort_seeds_for_test};

#[test]
fn sorted_disjoint_sources_use_boundary_then_source_ids() {
    let seeds = wave_seeds(
        &[square(220, 230), square(20, 30)],
        &[square(0, 100), square(200, 300)],
        1.0,
        true,
        CoordinateScale::Normal,
    )
    .unwrap();
    let ids = seeds
        .iter()
        .map(|seed| (seed.boundary, seed.src))
        .collect::<Vec<_>>();
    assert_eq!(ids, vec![(0, 1), (1, 0)]);
}
#[test]
fn equal_key_group_over_32_uses_fixed_msvc_permutation() {
    let mut seeds = (0..42)
        .map(|identity| WaveSeed {
            src: 7,
            boundary: if [0, 7, 14].contains(&identity) {
                2
            } else if [21, 28, 35].contains(&identity) {
                4
            } else {
                3
            },
            path: polygon(&[(identity, 0), (identity, 1)]),
        })
        .collect::<Vec<_>>();
    sort_seeds_for_test(&mut seeds);
    let actual = seeds
        .iter()
        .map(|seed| seed.path.points()[0].x() as usize)
        .collect::<Vec<_>>();
    assert_eq!(
        actual,
        vec![
            0, 7, 14, 1, 2, 3, 4, 5, 6, 8, 9, 10, 11, 12, 13, 15, 16, 17, 18, 19, 20, 26, 22, 23,
            24, 25, 27, 29, 30, 31, 32, 33, 34, 36, 37, 38, 39, 40, 41, 21, 28, 35,
        ]
    );
}
#[test]
fn unsorted_mode_preserves_discovery_order() {
    let seeds = wave_seeds(
        &[square(220, 230), square(20, 30)],
        &[square(0, 100), square(200, 300)],
        1.0,
        false,
        CoordinateScale::Normal,
    )
    .unwrap();
    assert_eq!(
        seeds
            .iter()
            .map(|seed| (seed.boundary, seed.src))
            .collect::<Vec<_>>(),
        vec![(1, 0), (0, 1)]
    );
}
#[test]
fn comparator_equivalent_groups_over_32_use_literal_nonstable_permutation() {
    const KEYS: [u32; 42] = [
        3, 9, 4, 10, 5, 0, 6, 1, 7, 2, 8, 3, 9, 4, 10, 5, 0, 6, 1, 7, 2, 8, 3, 9, 4, 10, 5, 0, 6,
        1, 7, 2, 8, 3, 9, 4, 10, 5, 0, 6, 1, 7,
    ];
    const EXPECTED: [usize; 42] = [
        5, 38, 27, 16, 40, 7, 29, 18, 9, 31, 20, 22, 0, 33, 11, 2, 35, 13, 24, 4, 15, 26, 37, 17,
        28, 6, 39, 30, 8, 41, 19, 21, 10, 32, 23, 12, 34, 1, 25, 14, 3, 36,
    ];
    let mut seeds = KEYS
        .iter()
        .enumerate()
        .map(|(identity, &boundary)| WaveSeed {
            src: 0,
            boundary,
            path: polygon(&[(identity as i64, 0), (identity as i64, 1)]),
        })
        .collect::<Vec<_>>();
    sort_seeds_for_test(&mut seeds);
    assert_eq!(
        seeds
            .iter()
            .map(|seed| seed.path.points()[0].x() as usize)
            .collect::<Vec<_>>(),
        EXPECTED
    );
    assert_ne!(&EXPECTED[..4], &[5, 16, 27, 38]);
}
type OrderedSeedPath = (u32, u32, Vec<(i64, i64)>);

fn ordered_seed_paths(seeds: &[WaveSeed]) -> Vec<OrderedSeedPath> {
    seeds
        .iter()
        .map(|seed| {
            (
                seed.src,
                seed.boundary,
                seed.path
                    .points()
                    .iter()
                    .map(|point| (point.x(), point.y()))
                    .collect(),
            )
        })
        .collect()
}

#[test]
fn pinned_clipperz_oracle_matches_expanded_and_final_ordered_paths() {
    let boundary = square(0, 100);
    let inside = square(20, 30);
    let (expanded, end) =
        expanded_source_paths_for_test(std::slice::from_ref(&inside), 1.0, 2).unwrap();
    assert_eq!(end, 3);
    assert_eq!(
        expanded[0]
            .iter()
            .map(|point| (point.x(), point.y(), point.z))
            .collect::<Vec<_>>(),
        vec![
            (31, 20, 2),
            (31, 30, 2),
            (30, 31, 2),
            (20, 31, 2),
            (19, 30, 2),
            (19, 20, 2),
            (20, 19, 2),
            (30, 19, 2),
            (31, 20, 2),
        ]
    );
    assert_eq!(
        ordered_seed_paths(
            &wave_seeds(
                &[inside],
                std::slice::from_ref(&boundary),
                1.0,
                false,
                CoordinateScale::Normal
            )
            .unwrap()
        ),
        vec![(
            0,
            0,
            vec![
                (31, 20),
                (31, 30),
                (30, 31),
                (20, 31),
                (19, 30),
                (19, 20),
                (20, 19),
                (30, 19),
                (31, 20)
            ]
        )]
    );

    let crossing = expolygon(&[(-20, 40), (120, 40), (120, 60), (-20, 60)], Vec::new());
    assert_eq!(
        ordered_seed_paths(
            &wave_seeds(
                &[crossing],
                std::slice::from_ref(&boundary),
                1.0,
                false,
                CoordinateScale::Normal
            )
            .unwrap()
        ),
        vec![
            (0, 0, vec![(100, 61), (0, 61)]),
            (0, 0, vec![(0, 39), (100, 39)]),
        ]
    );

    let holed_boundary = expolygon(
        &[(0, 0), (100, 0), (100, 100), (0, 100)],
        vec![polygon(&[(40, 40), (40, 60), (60, 60), (60, 40)])],
    );
    let across_hole = expolygon(&[(35, 45), (65, 45), (65, 55), (35, 55)], Vec::new());
    assert_eq!(
        ordered_seed_paths(
            &wave_seeds(
                &[across_hole],
                &[holed_boundary],
                1.0,
                false,
                CoordinateScale::Normal
            )
            .unwrap()
        ),
        vec![
            (
                0,
                0,
                vec![(40, 56), (35, 56), (34, 55), (34, 45), (35, 44), (40, 44)]
            ),
            (
                0,
                0,
                vec![
                    (60, 44),
                    (65, 44),
                    (66, 45),
                    (66, 45),
                    (66, 55),
                    (65, 56),
                    (60, 56)
                ]
            ),
        ]
    );

    let split = expolygon(&[(50, 50), (120, 40), (120, 60)], Vec::new());
    assert_eq!(
        ordered_seed_paths(
            &wave_seeds(&[split], &[boundary], 1.0, false, CoordinateScale::Normal).unwrap()
        ),
        vec![(0, 0, vec![(100, 58), (49, 51), (49, 49), (100, 42)])]
    );
}

#[test]
fn pinned_clipperz_oracle_matches_multiple_ids_and_overlapping_fallback() {
    let sources = [square(220, 230), square(20, 30)];
    let boundaries = [square(0, 100), square(200, 300)];
    assert_eq!(
        ordered_seed_paths(
            &wave_seeds(&sources, &boundaries, 1.0, false, CoordinateScale::Normal,).unwrap()
        ),
        vec![
            (
                0,
                1,
                vec![
                    (231, 220),
                    (231, 230),
                    (230, 231),
                    (220, 231),
                    (219, 230),
                    (219, 220),
                    (220, 219),
                    (230, 219),
                    (231, 220),
                ],
            ),
            (
                1,
                0,
                vec![
                    (31, 20),
                    (31, 30),
                    (30, 31),
                    (20, 31),
                    (19, 30),
                    (19, 20),
                    (20, 19),
                    (30, 19),
                    (31, 20),
                ],
            ),
        ]
    );
    let overlapping = [square(0, 100), square(0, 100)];
    let seeds = wave_seeds(
        &[square(20, 30)],
        &overlapping,
        1.0,
        false,
        CoordinateScale::Normal,
    )
    .unwrap();
    assert_eq!(
        ordered_seed_paths(&seeds)[0],
        (
            0,
            0,
            vec![
                (31, 20),
                (31, 30),
                (30, 31),
                (20, 31),
                (19, 30),
                (19, 20),
                (20, 19),
                (30, 19),
                (31, 20)
            ]
        )
    );
}
