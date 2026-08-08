use crate::geometry::{
    ClipperError, CoordinateScale, ExPolygon, Point, Polygon, RegionExpansion,
    RegionExpansionParameters, expand_merge_expolygons, merge_expansions_into_expolygons,
    propagate_waves_from_sources,
};

const OUTSIDE: i64 = 0x4000_0000_0000_0000;

type PathSnapshot = Vec<(i64, i64)>;
type ExPolygonSnapshot = (PathSnapshot, Vec<PathSnapshot>);
type ExpandMergeResult = Result<Vec<ExPolygon>, ClipperError>;
type ExpandMergeFn = fn(
    Vec<ExPolygon>,
    &[ExPolygon],
    &RegionExpansionParameters,
    CoordinateScale,
) -> ExpandMergeResult;

const EXPAND_MERGE: ExpandMergeFn = expand_merge_expolygons;

fn polygon(points: &[(i64, i64)]) -> Polygon {
    Polygon::new(points.iter().map(|&(x, y)| Point::new(x, y)).collect())
}

fn expolygon(contour: &[(i64, i64)], holes: Vec<Polygon>) -> ExPolygon {
    ExPolygon::new(polygon(contour), holes)
}

fn square(min: i64, max: i64) -> ExPolygon {
    expolygon(
        &[(min, min), (max, min), (max, max), (min, max)],
        Vec::new(),
    )
}

fn ordered_sources_with_holes() -> Vec<ExPolygon> {
    vec![
        expolygon(
            &[(0, 0), (100, 0), (100, 100), (0, 100)],
            vec![
                polygon(&[(10, 10), (10, 30), (30, 30), (30, 10)]),
                polygon(&[(60, 60), (60, 90), (90, 90), (90, 60)]),
            ],
        ),
        expolygon(
            &[(200, 0), (300, 0), (300, 100), (200, 100)],
            vec![polygon(&[(220, 20), (220, 80), (280, 80), (280, 20)])],
        ),
    ]
}

fn params() -> RegionExpansionParameters {
    RegionExpansionParameters {
        tiny_expansion: 1.0,
        initial_step: 2.0,
        other_step: 2.0,
        num_other_steps: 0,
        max_inflation: 4.0,
        arc_tolerance: 0.25,
        shortest_edge_length: 0.0,
    }
}

fn snapshot(expolygons: &[ExPolygon]) -> Vec<ExPolygonSnapshot> {
    expolygons
        .iter()
        .map(|expolygon| {
            (
                path_snapshot(expolygon.contour()),
                expolygon.holes().iter().map(path_snapshot).collect(),
            )
        })
        .collect()
}

fn path_snapshot(polygon: &Polygon) -> PathSnapshot {
    polygon
        .points()
        .iter()
        .map(|point| (point.x(), point.y()))
        .collect()
}

fn explicit_pipeline(
    src: Vec<ExPolygon>,
    boundary: &[ExPolygon],
    params: &RegionExpansionParameters,
    scale: CoordinateScale,
) -> ExpandMergeResult {
    let expanded = propagate_waves_from_sources(&src, boundary, params, scale)?;
    merge_expansions_into_expolygons(src, expanded, scale)
}

#[test]
fn task22o34_empty_and_no_expansion_sources_preserve_preconditions_and_ownership() {
    let params = params();
    assert_eq!(
        EXPAND_MERGE(Vec::new(), &[], &params, CoordinateScale::Normal),
        Ok(Vec::new())
    );

    let sources = ordered_sources_with_holes();
    let expected = snapshot(&sources);
    let pointers = sources
        .iter()
        .map(|source| {
            (
                source.contour().points().as_ptr(),
                source
                    .holes()
                    .iter()
                    .map(|hole| hole.points().as_ptr())
                    .collect::<Vec<_>>(),
            )
        })
        .collect::<Vec<_>>();
    let actual = EXPAND_MERGE(sources, &[], &params, CoordinateScale::Normal).unwrap();
    assert_eq!(snapshot(&actual), expected);
    for (source, (contour, holes)) in actual.iter().zip(pointers) {
        assert_eq!(source.contour().points().as_ptr(), contour);
        assert_eq!(
            source
                .holes()
                .iter()
                .map(|hole| hole.points().as_ptr())
                .collect::<Vec<_>>(),
            holes
        );
    }
    let explicit = explicit_pipeline(
        ordered_sources_with_holes(),
        &[],
        &params,
        CoordinateScale::Normal,
    )
    .unwrap();
    assert_eq!(snapshot(&explicit), expected);

    let invalid = RegionExpansionParameters {
        tiny_expansion: 0.0,
        ..params
    };
    assert!(
        std::panic::catch_unwind(|| {
            EXPAND_MERGE(Vec::new(), &[], &invalid, CoordinateScale::Normal)
        })
        .is_err()
    );
}

#[test]
fn task22o34_natural_merge_matches_complete_literal_and_explicit_pipeline_at_both_scales() {
    let expected = vec![(
        vec![
            (37, 10),
            (40, 10),
            (40, 13),
            (43, 16),
            (43, 34),
            (40, 37),
            (40, 40),
            (37, 40),
            (34, 43),
            (16, 43),
            (13, 40),
            (10, 40),
            (10, 37),
            (7, 34),
            (7, 16),
            (10, 13),
            (10, 10),
            (13, 10),
            (16, 7),
            (34, 7),
        ],
        Vec::new(),
    )];
    let params = params();

    for scale in [CoordinateScale::Normal, CoordinateScale::LargeBed] {
        let boundary = [square(0, 100)];
        let actual = EXPAND_MERGE(vec![square(20, 30)], &boundary, &params, scale).unwrap();
        let explicit = explicit_pipeline(vec![square(20, 30)], &boundary, &params, scale).unwrap();
        assert_eq!(snapshot(&actual), expected);
        assert_eq!(snapshot(&actual), snapshot(&explicit));
    }
}

#[test]
fn task22o34_discovery_error_precedes_the_merge_empty_contour_panic() {
    let empty = ExPolygon::new(Polygon::new(Vec::new()), Vec::new());
    let later_merge = std::panic::catch_unwind(|| {
        merge_expansions_into_expolygons(
            vec![ExPolygon::new(Polygon::new(Vec::new()), Vec::new())],
            vec![RegionExpansion {
                polygon: polygon(&[(0, 0), (100, 0), (100, 100), (0, 100)]),
                src_id: 0,
                boundary_id: 0,
            }],
            CoordinateScale::Normal,
        )
    });
    assert!(later_merge.is_err());

    let invalid_boundary = [expolygon(
        &[(OUTSIDE, 0), (OUTSIDE, 10), (OUTSIDE - 1, 10)],
        Vec::new(),
    )];
    assert_eq!(
        EXPAND_MERGE(
            vec![empty, square(20, 30)],
            &invalid_boundary,
            &params(),
            CoordinateScale::Normal,
        ),
        Err(ClipperError::CoordinateOutOfRange)
    );
}

#[test]
fn task22o34_propagation_error_escapes_before_merge() {
    let boundary = [square(0, 1000)];
    let source = vec![square(200, 300)];
    let params = RegionExpansionParameters {
        other_step: OUTSIDE as f32,
        num_other_steps: 1,
        max_inflation: 500.0,
        arc_tolerance: f64::MAX,
        shortest_edge_length: 0.1,
        ..params()
    };
    assert_eq!(
        EXPAND_MERGE(source, &boundary, &params, CoordinateScale::Normal),
        Err(ClipperError::CoordinateOutOfRange)
    );
}

#[test]
fn task22o34_successful_nonempty_propagation_reaches_merge() {
    let boundary = [square(0, 100)];
    let params = params();
    let source = vec![square(20, 30)];
    let expanded =
        propagate_waves_from_sources(&source, &boundary, &params, CoordinateScale::Normal).unwrap();
    assert!(!expanded.is_empty());
    let expected =
        merge_expansions_into_expolygons(source, expanded, CoordinateScale::Normal).unwrap();

    let actual = EXPAND_MERGE(
        vec![square(20, 30)],
        &boundary,
        &params,
        CoordinateScale::Normal,
    )
    .unwrap();
    assert_eq!(snapshot(&actual), snapshot(&expected));
}
