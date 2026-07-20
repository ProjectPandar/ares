use crate::geometry::{
    ExPolygon, Point, Polygon,
    clipper::{ClipperError, variable_offset_inner_ex},
};

type Geometry = Vec<(Vec<(i64, i64)>, Vec<Vec<(i64, i64)>>)>;

#[test]
fn task22m_variable_offset_uses_each_contour_vertex_delta() {
    let input = expolygon(&[(0, 0), (100, 0), (100, 100), (0, 100)], Vec::new());

    let uniform =
        variable_offset_inner_ex(&input, &[vec![-10.0, -10.0, -10.0, -10.0]], 2.0).unwrap();
    assert_eq!(
        geometry(&uniform),
        vec![(vec![(90, 90), (10, 90), (10, 10), (90, 10)], Vec::new(),)]
    );

    let variable =
        variable_offset_inner_ex(&input, &[vec![-10.0, -20.0, -30.0, -40.0]], 2.0).unwrap();
    assert_eq!(
        geometry(&variable),
        vec![(vec![(78, 18), (73, 67), (29, 63), (13, 11)], Vec::new(),)]
    );
}

#[test]
fn task22m_variable_offset_expands_holes_and_preserves_ownership() {
    let input = expolygon(
        &[(0, 0), (100, 0), (100, 100), (0, 100)],
        vec![vec![(30, 30), (30, 70), (70, 70), (70, 30)]],
    );
    let output = variable_offset_inner_ex(
        &input,
        &[
            vec![-10.0, -10.0, -10.0, -10.0],
            vec![-5.0, -5.0, -5.0, -5.0],
        ],
        2.0,
    )
    .unwrap();

    assert_eq!(
        geometry(&output),
        vec![(
            vec![(90, 90), (10, 90), (10, 10), (90, 10)],
            vec![vec![(25, 25), (25, 75), (75, 75), (75, 25)]],
        )]
    );
}

#[test]
fn task22m_variable_offset_preserves_concave_and_short_edge_semantics() {
    let concave = expolygon(
        &[(0, 0), (100, 0), (100, 40), (40, 40), (40, 100), (0, 100)],
        Vec::new(),
    );
    let concave_output =
        variable_offset_inner_ex(&concave, &[vec![-2.0, -4.0, -6.0, -8.0, -10.0, -12.0]], 2.0)
            .unwrap();
    assert_eq!(
        geometry(&concave_output),
        vec![(
            vec![(96, 4), (94, 34), (32, 32), (30, 90), (11, 89), (2, 2)],
            Vec::new(),
        )]
    );

    let short_edge = expolygon(
        &[(0, 0), (3, 0), (10_000, 0), (10_000, 10_000), (0, 10_000)],
        Vec::new(),
    );
    let short_output = variable_offset_inner_ex(
        &short_edge,
        &[vec![-1_000.0, -2_000.0, -3_000.0, -4_000.0, -5_000.0]],
        2.0,
    )
    .unwrap();
    assert_eq!(
        geometry(&short_output),
        vec![(
            vec![
                (6_765, 2_353),
                (6_436, 5_644),
                (3_125, 5_313),
                (1_522, 1_304)
            ],
            Vec::new(),
        )]
    );
}

#[test]
fn task22m_variable_offset_preserves_collinear_vertices_and_miter_limits() {
    let collinear = expolygon(
        &[(0, 0), (50, 0), (100, 0), (100, 100), (0, 100)],
        Vec::new(),
    );
    let collinear_output =
        variable_offset_inner_ex(&collinear, &[vec![-10.0, -20.0, -10.0, -10.0, -10.0]], 2.0)
            .unwrap();
    assert_eq!(
        geometry(&collinear_output),
        vec![(
            vec![(90, 90), (10, 90), (10, 12), (50, 20), (90, 12)],
            Vec::new(),
        )]
    );

    let notch = expolygon(
        &[
            (0, 0),
            (1_000, 0),
            (1_000, 1_000),
            (600, 1_000),
            (500, 800),
            (400, 1_000),
            (0, 1_000),
        ],
        Vec::new(),
    );
    let limited = vec![(
        vec![
            (950, 950),
            (631, 950),
            (531, 750),
            (469, 750),
            (369, 950),
            (50, 950),
            (50, 50),
            (950, 50),
        ],
        Vec::new(),
    )];
    for miter_limit in [1.0, 2.0] {
        assert_eq!(
            geometry(&variable_offset_inner_ex(&notch, &[vec![-50.0; 7]], miter_limit).unwrap()),
            limited
        );
    }
    assert_eq!(
        geometry(&variable_offset_inner_ex(&notch, &[vec![-50.0; 7]], 4.0).unwrap()),
        vec![(
            vec![
                (950, 950),
                (631, 950),
                (500, 688),
                (369, 950),
                (50, 950),
                (50, 50),
                (950, 50),
            ],
            Vec::new(),
        )]
    );
}

#[test]
fn task22m_variable_offset_returns_split_and_empty_results_without_fallback() {
    let split = expolygon(
        &[
            (0, 0),
            (100, 0),
            (100, 45),
            (200, 45),
            (200, 0),
            (300, 0),
            (300, 100),
            (200, 100),
            (200, 55),
            (100, 55),
            (100, 100),
            (0, 100),
        ],
        Vec::new(),
    );
    assert_eq!(
        geometry(&variable_offset_inner_ex(&split, &[vec![-10.0; 12]], 2.0).unwrap()),
        vec![
            (vec![(290, 10), (290, 90), (210, 90), (210, 10)], Vec::new()),
            (vec![(10, 90), (10, 10), (90, 10), (90, 90)], Vec::new()),
        ]
    );

    let eroded = expolygon(&[(0, 0), (100, 0), (100, 100), (0, 100)], Vec::new());
    assert!(
        variable_offset_inner_ex(&eroded, &[vec![-60.0; 4]], 2.0)
            .unwrap()
            .is_empty()
    );
    assert!(
        variable_offset_inner_ex(&expolygon(&[], Vec::new()), &[vec![]], 2.0)
            .unwrap()
            .is_empty()
    );
    assert!(
        variable_offset_inner_ex(
            &expolygon(&[(0, 0), (1, 0)], Vec::new()),
            &[vec![-1.0; 2]],
            2.0,
        )
        .unwrap()
        .is_empty()
    );
}

#[test]
fn task22m_variable_offset_preserves_scale_and_half_away_rounding() {
    for (contour, delta, expected) in [
        (
            vec![
                (0, 0),
                (20_000_000, 0),
                (20_000_000, 12_000_000),
                (0, 12_000_000),
            ],
            -150_000.0,
            vec![
                (19_850_000, 11_850_000),
                (150_000, 11_850_000),
                (150_000, 150_000),
                (19_850_000, 150_000),
            ],
        ),
        (
            vec![
                (0, 0),
                (2_000_000, 0),
                (2_000_000, 1_200_000),
                (0, 1_200_000),
            ],
            -15_000.0,
            vec![
                (1_985_000, 1_185_000),
                (15_000, 1_185_000),
                (15_000, 15_000),
                (1_985_000, 15_000),
            ],
        ),
    ] {
        let output =
            variable_offset_inner_ex(&expolygon(&contour, Vec::new()), &[vec![delta; 4]], 2.0)
                .unwrap();
        assert_eq!(geometry(&output), vec![(expected, Vec::new())]);
    }

    let negative = expolygon(&[(-10, -10), (0, -10), (0, 0), (-10, 0)], Vec::new());
    assert_eq!(
        geometry(&variable_offset_inner_ex(&negative, &[vec![-0.5; 4]], 2.0).unwrap()),
        vec![(vec![(-1, -1), (-10, -1), (-10, -10), (-1, -10)], Vec::new(),)]
    );
}

#[test]
fn task22m_variable_offset_propagates_clipper_coordinate_errors() {
    let input = expolygon(
        &[
            (i64::MAX - 100, i64::MAX - 100),
            (i64::MAX, i64::MAX - 100),
            (i64::MAX, i64::MAX),
            (i64::MAX - 100, i64::MAX),
        ],
        Vec::new(),
    );

    assert_eq!(
        variable_offset_inner_ex(&input, &[vec![-10.0, -10.0, -10.0, -10.0]], 2.0,),
        Err(ClipperError::CoordinateOutOfRange)
    );

    let finite = expolygon(&[(0, 0), (100, 0), (100, 100), (0, 100)], Vec::new());
    assert_eq!(
        variable_offset_inner_ex(&finite, &[vec![f32::NAN, -10.0, -10.0, -10.0]], 2.0,),
        Err(ClipperError::CoordinateOutOfRange)
    );
}

fn expolygon(contour: &[(i64, i64)], holes: Vec<Vec<(i64, i64)>>) -> ExPolygon {
    ExPolygon::new(
        polygon(contour),
        holes.iter().map(|hole| polygon(hole)).collect(),
    )
}

fn polygon(points: &[(i64, i64)]) -> Polygon {
    Polygon::new(points.iter().map(|&(x, y)| Point::new(x, y)).collect())
}

fn geometry(expolygons: &[ExPolygon]) -> Geometry {
    expolygons
        .iter()
        .map(|expolygon| {
            (
                coordinates(expolygon.contour()),
                expolygon.holes().iter().map(coordinates).collect(),
            )
        })
        .collect()
}

fn coordinates(polygon: &Polygon) -> Vec<(i64, i64)> {
    polygon
        .points()
        .iter()
        .map(|point| (point.x(), point.y()))
        .collect()
}
