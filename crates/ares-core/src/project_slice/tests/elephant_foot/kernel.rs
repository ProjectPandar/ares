use crate::geometry::{ClipperError, Coord, CoordinateScale, ExPolygon, Point, Polygon};

use super::super::super::elephant_foot::{
    compensate_expolygon, compensate_expolygons, derive_geometry,
};

fn polygon(values: &[(Coord, Coord)]) -> Polygon {
    Polygon::new(values.iter().map(|&(x, y)| Point::new(x, y)).collect())
}

fn expolygon(contour: &[(Coord, Coord)], holes: &[&[(Coord, Coord)]]) -> ExPolygon {
    ExPolygon::new(
        polygon(contour),
        holes.iter().map(|hole| polygon(hole)).collect(),
    )
}

fn rectangle(min_x: Coord, min_y: Coord, max_x: Coord, max_y: Coord) -> ExPolygon {
    expolygon(
        &[
            (min_x, min_y),
            (max_x, min_y),
            (max_x, max_y),
            (min_x, max_y),
        ],
        &[],
    )
}

#[test]
fn task22m_elephant_foot_tiny_predicates_are_independent_and_strict() {
    let geometry = derive_geometry(0.7, 0.15, CoordinateScale::Normal).unwrap();

    let width_equal = rectangle(0, 0, 1_000_100, 6_000_000);
    assert_eq!(
        compensate_expolygon(&width_equal, geometry),
        Ok(expolygon(
            &[
                (850_100, 5_850_000),
                (150_000, 5_850_000),
                (150_000, 150_000),
                (850_100, 150_000),
            ],
            &[],
        ))
    );

    let height_equal = rectangle(0, 0, 6_000_000, 1_000_100);
    assert_eq!(
        compensate_expolygon(&height_equal, geometry),
        Ok(expolygon(
            &[
                (5_850_000, 850_100),
                (150_000, 850_100),
                (150_000, 150_000),
                (5_850_000, 150_000),
            ],
            &[],
        ))
    );

    let area_equal = rectangle(0, 0, 2_000_000, 2_500_000);
    assert_eq!(
        compensate_expolygon(&area_equal, geometry),
        Ok(expolygon(
            &[
                (1_850_000, 2_350_000),
                (150_000, 2_350_000),
                (150_000, 150_000),
                (1_850_000, 150_000),
            ],
            &[],
        ))
    );
}

#[test]
fn task22m_elephant_foot_tiny_area_uses_signed_holes_and_contour_bbox() {
    let geometry = derive_geometry(0.7, 0.15, CoordinateScale::Normal).unwrap();
    let area_hole = [
        (975_000, 975_000),
        (975_000, 1_275_000),
        (1_275_000, 1_275_000),
        (1_275_000, 975_000),
    ];
    let area_tiny = expolygon(
        &[
            (0, 0),
            (2_250_000, 0),
            (2_250_000, 2_250_000),
            (0, 2_250_000),
        ],
        &[&area_hole],
    );
    assert_eq!(
        compensate_expolygon(&area_tiny, geometry),
        Ok(area_tiny.clone())
    );

    let small_hole = [
        (1_750_000, 1_750_000),
        (1_750_000, 2_250_000),
        (2_250_000, 2_250_000),
        (2_250_000, 1_750_000),
    ];
    let with_small_hole = expolygon(
        &[
            (0, 0),
            (4_000_000, 0),
            (4_000_000, 4_000_000),
            (0, 4_000_000),
        ],
        &[&small_hole],
    );
    let compensated = compensate_expolygon(&with_small_hole, geometry).unwrap();
    assert_eq!(
        compensated.contour(),
        &polygon(&[
            (3_850_000, 3_850_000),
            (150_000, 3_850_000),
            (150_000, 150_000),
            (3_850_000, 150_000),
        ])
    );
    assert_ne!(compensated, with_small_hole);
}

#[test]
fn task22m_elephant_foot_propagates_per_item_range_errors_before_union() {
    const BASE: Coord = 0x4000_0000_0000_0000;
    let translated = rectangle(BASE, 0, BASE + 20_000_000, 12_000_000);
    let geometry = derive_geometry(
        f64::from(f32::from_bits(0x3f75_032c)),
        0.15,
        CoordinateScale::Normal,
    )
    .unwrap();
    assert_eq!(
        compensate_expolygon(&translated, geometry),
        Err(ClipperError::CoordinateOutOfRange)
    );
}

#[test]
fn task22m_elephant_foot_compensates_each_input_before_union() {
    let input = [
        rectangle(0, 0, 10_000_000, 10_000_000),
        rectangle(9_800_000, 0, 19_800_000, 10_000_000),
    ];
    assert_eq!(
        compensate_expolygons(
            &input,
            f64::from(f32::from_bits(0x3f75_032c)),
            0.15,
            CoordinateScale::Normal,
        ),
        Ok(vec![
            expolygon(
                &[
                    (9_850_000, 9_850_000),
                    (150_000, 9_850_000),
                    (150_000, 150_000),
                    (9_850_000, 150_000),
                ],
                &[],
            ),
            expolygon(
                &[
                    (19_650_000, 9_850_000),
                    (9_950_000, 9_850_000),
                    (9_950_000, 150_000),
                    (19_650_000, 150_000),
                ],
                &[],
            ),
        ])
    );
}

#[test]
fn task22m_elephant_foot_large_scale_preserves_the_f32_roundtrip() {
    let input = rectangle(0, 0, 31_010, 200_000);
    let staged = f64::from_bits(0x3fc3_3333_3333_3334);
    let geometry = derive_geometry(0.01, staged, CoordinateScale::LargeBed).unwrap();
    assert_eq!(compensate_expolygon(&input, geometry), Ok(input.clone()));

    let direct = derive_geometry(0.01, 0.15, CoordinateScale::LargeBed).unwrap();
    assert_eq!(
        compensate_expolygon(&input, direct),
        Ok(expolygon(
            &[
                (16_010, 185_000),
                (15_000, 185_000),
                (15_000, 15_000),
                (16_010, 15_000),
            ],
            &[],
        ))
    );
}

#[test]
fn task22m_elephant_foot_area_threshold_preserves_source_operation_order() {
    let input = rectangle(0, 0, 1_404_956, 1_744_694);
    assert_eq!(
        compensate_expolygons(
            &[input],
            f64::from(f32::from_bits(0x3ecc_e39c)),
            0.15,
            CoordinateScale::Normal,
        ),
        Ok(vec![expolygon(
            &[
                (1_254_956, 1_594_694),
                (150_000, 1_594_694),
                (150_000, 150_000),
                (1_254_956, 150_000),
            ],
            &[],
        )])
    );
}
