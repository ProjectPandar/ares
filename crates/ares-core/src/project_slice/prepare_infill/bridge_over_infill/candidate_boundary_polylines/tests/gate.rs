use super::*;

#[test]
fn task22o59_empty_survivor_gate_ignores_scalars_geometry_and_engine() {
    let area = candidate_area(Vec::new(), vec![outside_range()]);
    let total = [outside_range()];

    assert_eq!(
        prepare_candidate_boundary_polylines(&area, &total, i64::MIN, f32::NAN).unwrap(),
        None
    );
    assert_eq!(
        prepare_candidate_boundary_polylines_using(
            operation_input(&area, &total, 0, f32::NEG_INFINITY),
            |_, _| panic!("offset must not run after the source empty gate"),
        )
        .unwrap(),
        None
    );
}

#[test]
fn task22o59_gate_uses_survivors_not_limiting_or_total_emptiness() {
    let area = candidate_area(vec![rectangle(0, 0, 1, 1)], Vec::new());
    let calls = std::cell::Cell::new(0);
    let output =
        prepare_candidate_boundary_polylines_using(operation_input(&area, &[], 1, 1.0), |_, _| {
            calls.set(calls.get() + 1);
            Ok(Vec::new())
        })
        .unwrap();

    assert_eq!(output, Some(Vec::new()));
    assert_eq!(calls.get(), 2);
}
