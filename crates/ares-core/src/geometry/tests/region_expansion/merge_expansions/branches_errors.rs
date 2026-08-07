use super::*;

const OUTSIDE: i64 = 0x4000_0000_0000_0000;

fn missing_sample_inputs() -> (Vec<ExPolygon>, Vec<RegionExpansion>) {
    (
        vec![expolygon(&[(0, 0), (10, 0), (20, 0)], Vec::new())],
        vec![expansion(0, 0, &[(30, 0), (40, 0), (50, 0)])],
    )
}

#[cfg(debug_assertions)]
#[test]
#[should_panic]
fn task22o33_missing_multi_result_sample_is_a_debug_assertion() {
    let (src, expanded) = missing_sample_inputs();
    let _ = MERGE(src, expanded, CoordinateScale::Normal);
}

#[cfg(not(debug_assertions))]
#[test]
fn task22o33_missing_multi_result_sample_is_omitted_in_release() {
    let (src, expanded) = missing_sample_inputs();
    assert!(
        MERGE(src, expanded, CoordinateScale::Normal)
            .unwrap()
            .is_empty()
    );
}

#[test]
fn task22o33_zero_safety_union_result_emits_no_expolygon() {
    let source = expolygon(&[(0, 0)], Vec::new());
    let expanded = vec![expansion(0, 0, &[(20, 0)])];

    assert!(
        MERGE(vec![source], expanded, CoordinateScale::Normal)
            .unwrap()
            .is_empty()
    );
}

#[test]
#[should_panic]
fn task22o33_empty_expanded_source_contour_is_a_trusted_panic() {
    let source = ExPolygon::new(Polygon::new(Vec::new()), Vec::new());
    let expanded = vec![expansion(0, 0, &[(0, 0), (100, 0), (100, 100), (0, 100)])];
    let _ = MERGE(vec![source], expanded, CoordinateScale::Normal);
}

#[test]
#[should_panic]
fn task22o33_malformed_source_id_is_a_trusted_panic() {
    let expanded = vec![expansion(1, 0, &[(0, 0), (100, 0), (100, 100), (0, 100)])];
    let _ = MERGE(vec![square(0, 100)], expanded, CoordinateScale::Normal);
}

#[test]
fn task22o33_safety_offset_error_escapes_directly() {
    let expanded = vec![expansion(
        0,
        0,
        &[(OUTSIDE, 0), (OUTSIDE, 10), (OUTSIDE - 1, 10)],
    )];
    assert_eq!(
        MERGE(vec![square(0, 100)], expanded, CoordinateScale::Normal),
        Err(ClipperError::CoordinateOutOfRange)
    );
}

#[test]
fn task22o33_sorted_first_group_error_precedes_later_trusted_panic() {
    let sources = vec![
        square(0, 100),
        ExPolygon::new(Polygon::new(Vec::new()), Vec::new()),
    ];
    let expanded = vec![
        expansion(1, 2, &[(200, 0), (300, 0), (300, 100), (200, 100)]),
        expansion(0, 1, &[(OUTSIDE, 0), (OUTSIDE, 10), (OUTSIDE - 1, 10)]),
    ];

    let result = std::panic::catch_unwind(|| MERGE(sources, expanded, CoordinateScale::Normal))
        .expect("the sorted first group error must precede later source access");
    assert_eq!(result, Err(ClipperError::CoordinateOutOfRange));
}

#[test]
fn task22o33_unexpanded_sources_bypass_clipper_validation() {
    let invalid = expolygon(
        &[(OUTSIDE, 0), (OUTSIDE, 10), (OUTSIDE - 1, 10)],
        Vec::new(),
    );
    let pointer = invalid.contour().points().as_ptr();

    let actual = MERGE(
        vec![invalid, square(0, 100)],
        Vec::new(),
        CoordinateScale::Normal,
    )
    .unwrap();
    assert_eq!(actual[0].contour().points().as_ptr(), pointer);
}
