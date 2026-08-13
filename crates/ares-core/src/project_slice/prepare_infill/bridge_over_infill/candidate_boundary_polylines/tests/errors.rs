use super::*;

#[test]
fn task22o59_natural_offset_errors_preserve_source_precedence() {
    let survivor = vec![rectangle(0, 0, 1, 1)];
    let invalid_total = [outside_range()];
    let invalid_limiting = vec![outside_range()];
    let first = candidate_area(survivor.clone(), invalid_limiting.clone());
    assert_range_error(prepare_candidate_boundary_polylines(
        &first,
        &invalid_total,
        1,
        1.0,
    ));

    let second = candidate_area(survivor, invalid_limiting);
    assert_range_error(prepare_candidate_boundary_polylines(
        &second,
        &[rectangle(0, 0, 10, 10)],
        1,
        1.0,
    ));
}

#[test]
fn task22o59_injected_competing_errors_keep_total_before_limiting() {
    let area = candidate_area(vec![rectangle(0, 0, 1, 1)], vec![rectangle(30, 0, 40, 10)]);
    let total = [rectangle(0, 0, 10, 10)];

    for fail_at in 0..=1 {
        let calls = std::cell::Cell::new(0);
        let result = prepare_candidate_boundary_polylines_using(
            operation_input(&area, &total, 1, 1.0),
            |subject, _| {
                let call = calls.get();
                calls.set(call + 1);
                if call == fail_at {
                    Err(ClipperError::CoordinateOutOfRange)
                } else {
                    Ok(subject.to_vec())
                }
            },
        );
        assert_range_error(result);
        assert_eq!(calls.get(), fail_at + 1);
    }
}
