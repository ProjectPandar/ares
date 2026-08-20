use crate::geometry::{
    ClipperError, CoordinateScale, ExPolygon, Point, Polygon, variable_offset_inner_ex,
};

use super::super::super::elephant_foot::{
    compensate_expolygon, compensate_expolygons, derive_geometry, prepare_offset,
};

const DEFAULT_MINIMUM_WIDTH: f64 = f32::from_bits(0x3f75_032c) as f64;

fn polygon(points: &[(i64, i64)]) -> Polygon {
    Polygon::new(points.iter().map(|&(x, y)| Point::new(x, y)).collect())
}

fn expolygon(contour: &[(i64, i64)], holes: &[&[(i64, i64)]]) -> ExPolygon {
    ExPolygon::new(
        polygon(contour),
        holes.iter().map(|hole| polygon(hole)).collect(),
    )
}

fn rectangle(min_x: i64, min_y: i64, max_x: i64, max_y: i64) -> ExPolygon {
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

fn stage_compensation(scale: CoordinateScale) -> f64 {
    f64::from((0.15 / scale.factor()) as f32) * scale.factor()
}

fn compensate(input: &[ExPolygon], scale: CoordinateScale) -> Vec<ExPolygon> {
    compensate_expolygons(
        input,
        DEFAULT_MINIMUM_WIDTH,
        stage_compensation(scale),
        scale,
    )
    .unwrap()
}

#[test]
fn task22m_oracle_freezes_normal_and_large_scale_rectangles() {
    let normal_mm = stage_compensation(CoordinateScale::Normal);
    let large_mm = stage_compensation(CoordinateScale::LargeBed);
    assert_eq!(normal_mm.to_bits(), 0x3fc3_3333_3333_3333);
    assert_eq!(large_mm.to_bits(), 0x3fc3_3333_3333_3334);

    let normal = rectangle(0, 0, 20_000_000, 12_000_000);
    assert_eq!(
        compensate(&[normal], CoordinateScale::Normal),
        [expolygon(
            &[
                (19_850_000, 11_850_000),
                (150_000, 11_850_000),
                (150_000, 150_000),
                (19_850_000, 150_000),
            ],
            &[],
        )]
    );

    let large = rectangle(0, 0, 2_000_000, 1_200_000);
    assert_eq!(
        compensate(&[large], CoordinateScale::LargeBed),
        [expolygon(
            &[
                (1_985_000, 1_185_000),
                (15_000, 1_185_000),
                (15_000, 15_000),
                (1_985_000, 15_000),
            ],
            &[],
        )]
    );
}

#[test]
fn task22m_oracle_freezes_narrow_neck_and_hole() {
    let hole = [
        (500_000, 500_000),
        (500_000, 2_000_000),
        (2_000_000, 2_000_000),
        (2_000_000, 500_000),
    ];
    let input = expolygon(
        &[
            (0, 0),
            (8_000_000, 0),
            (8_000_000, 4_000_000),
            (4_350_000, 4_000_000),
            (4_350_000, 9_000_000),
            (3_650_000, 9_000_000),
            (3_650_000, 4_000_000),
            (0, 4_000_000),
        ],
        &[&hole],
    );
    let expected_hole = [
        (500_000, 500_000),
        (500_000, 2_000_000),
        (1_000_000, 2_100_219),
        (1_500_000, 2_141_900),
        (2_148_988, 2_148_988),
        (2_141_900, 1_500_000),
        (2_100_219, 1_000_000),
        (2_000_000, 500_000),
    ];
    let expected = expolygon(
        &[
            (3_000_000, 73_087),
            (3_500_000, 126_101),
            (4_000_000, 146_209),
            (4_500_000, 149_765),
            (5_000_000, 150_000),
            (7_850_000, 150_000),
            (7_850_000, 3_850_000),
            (6_175_000, 3_850_000),
            (5_718_750, 3_850_506),
            (5_262_500, 3_858_100),
            (4_806_250, 3_899_781),
            (4_350_000, 4_000_000),
            (4_350_000, 8_500_000),
            (4_272_788, 8_896_469),
            (4_000_000, 8_874_721),
            (3_727_212, 8_896_469),
            (3_650_000, 8_500_000),
            (3_650_000, 4_000_000),
            (3_193_750, 3_899_781),
            (2_737_500, 3_858_100),
            (2_281_250, 3_850_506),
            (1_825_000, 3_850_000),
            (912_500, 3_850_000),
            (456_250, 3_850_235),
            (140_285, 3_852_698),
            (126_101, 3_500_000),
            (73_087, 3_000_000),
            (0, 2_500_000),
            (0, 0),
            (2_500_000, 0),
        ],
        &[&expected_hole],
    );

    let output = compensate(&[input], CoordinateScale::Normal);
    assert_eq!(output, [expected]);
    assert!(output[0].contour().area() > 0.0);
    assert!(output[0].holes()[0].area() < 0.0);
}

#[test]
fn task22o228_batch_preserves_fallback_geometry_and_input_order() {
    let tiny = rectangle(0, 0, 1_000_000, 1_000_000);
    let left_hole = [(10, 10), (10, 50), (50, 50), (50, 10)];
    let right_hole = [(110, 10), (110, 50), (150, 50), (150, 10)];
    let input = vec![
        tiny,
        expolygon(&[(0, 0), (60, 0), (60, 60), (0, 60)], &[&left_hole]),
        rectangle(20, 20, 40, 40),
        expolygon(&[(100, 0), (160, 0), (160, 60), (100, 60)], &[&right_hole]),
    ];

    assert_eq!(compensate(&input, CoordinateScale::Normal), input);
}

#[test]
fn task22o228_batch_preserves_natural_result_count_fallbacks() {
    let minimum_width = f32::from_bits(0x3480_0000) as f64;
    let geometry = derive_geometry(minimum_width, 0.15, CoordinateScale::Normal).unwrap();
    let dumbbell = expolygon(
        &[
            (0, 0),
            (10_000_000, 0),
            (10_000_000, 4_950_000),
            (20_000_000, 4_950_000),
            (20_000_000, 0),
            (30_000_000, 0),
            (30_000_000, 10_000_000),
            (20_000_000, 10_000_000),
            (20_000_000, 5_050_000),
            (10_000_000, 5_050_000),
            (10_000_000, 10_000_000),
            (0, 10_000_000),
        ],
        &[],
    );
    let prepared = prepare_offset(&dumbbell, geometry).unwrap();
    let (shape, deltas) = prepared.as_parts();
    assert_eq!(
        variable_offset_inner_ex(shape, deltas, 2.0).unwrap().len(),
        2
    );
    assert_eq!(
        compensate_expolygon(&dumbbell, geometry),
        Ok(dumbbell.clone())
    );
    let batch = compensate_expolygons(
        std::slice::from_ref(&dumbbell),
        minimum_width,
        0.15,
        CoordinateScale::Normal,
    )
    .unwrap();
    assert_eq!(batch, [dumbbell]);

    let ring_hole = [
        (100_000, 100_000),
        (100_000, 19_900_000),
        (19_900_000, 19_900_000),
        (19_900_000, 100_000),
    ];
    let ring = expolygon(
        &[
            (0, 0),
            (20_000_000, 0),
            (20_000_000, 20_000_000),
            (0, 20_000_000),
        ],
        &[&ring_hole],
    );
    let prepared = prepare_offset(&ring, geometry).unwrap();
    let (shape, deltas) = prepared.as_parts();
    assert!(
        variable_offset_inner_ex(shape, deltas, 2.0)
            .unwrap()
            .is_empty()
    );
    assert_eq!(compensate_expolygon(&ring, geometry), Ok(ring.clone()));
    let batch = compensate_expolygons(
        std::slice::from_ref(&ring),
        minimum_width,
        0.15,
        CoordinateScale::Normal,
    )
    .unwrap();
    assert_eq!(batch, [ring]);
}

#[test]
fn task22m_elephant_foot_propagates_nonfinite_and_clipper_range_errors() {
    assert_eq!(
        derive_geometry(DEFAULT_MINIMUM_WIDTH, f64::MAX, CoordinateScale::Normal),
        Err(ClipperError::CoordinateOutOfRange)
    );

    const BASE: i64 = 0x4000_0000_0000_0000;
    let translated = rectangle(BASE, 0, BASE + 20_000_000, 12_000_000);
    assert_eq!(
        compensate_expolygons(
            &[translated],
            DEFAULT_MINIMUM_WIDTH,
            0.15,
            CoordinateScale::Normal,
        ),
        Err(ClipperError::CoordinateOutOfRange)
    );
}

#[test]
fn task22m_elephant_foot_preserves_closed_simplifier_seam_dependence() {
    let a = expolygon(
        &[
            (0, 0),
            (4_000_000, 0),
            (4_000_050, 50),
            (4_000_000, 4_000_000),
            (0, 4_000_000),
            (-50, 50),
        ],
        &[],
    );
    let b = expolygon(
        &[
            (4_000_050, 50),
            (4_000_000, 4_000_000),
            (0, 4_000_000),
            (-50, 50),
            (0, 0),
            (4_000_000, 0),
        ],
        &[],
    );
    let geometry = derive_geometry(DEFAULT_MINIMUM_WIDTH, 0.15, CoordinateScale::Normal).unwrap();
    assert_eq!(
        compensate_expolygon(&a, geometry),
        Ok(expolygon(
            &[
                (3_850_000, 3_850_000),
                (150_000, 3_850_000),
                (150_000, 150_000),
                (3_850_000, 150_000),
            ],
            &[],
        ))
    );
    assert_eq!(
        compensate_expolygon(&b, geometry),
        Ok(expolygon(
            &[
                (444_448, 150_005),
                (888_898, 150_011),
                (1_333_348, 150_016),
                (1_777_798, 150_022),
                (2_222_248, 150_027),
                (2_666_698, 150_033),
                (3_111_148, 150_038),
                (3_850_048, 150_048),
                (3_850_043, 500_041),
                (3_850_025, 2_000_023),
                (3_850_018, 2_500_016),
                (3_850_002, 3_850_000),
                (150_000, 3_850_000),
                (150_000, 150_002),
            ],
            &[],
        ))
    );
}
