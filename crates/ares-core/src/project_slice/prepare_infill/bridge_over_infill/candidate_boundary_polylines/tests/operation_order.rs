use super::*;

#[test]
fn task22o59_calls_total_then_limiting_once_and_appends_without_reordering() {
    let limiting = vec![rectangle(100, 0, 110, 10)];
    let area = candidate_area(vec![rectangle(0, 0, 1, 1)], limiting);
    let total = vec![rectangle(0, 0, 10, 10)];
    let total_result = vec![
        rectangle(70, 0, 80, 10),
        Polygon::new(vec![
            Point::new(10, 1),
            Point::new(18, 2),
            Point::new(15, 9),
        ]),
    ];
    let limiting_result = vec![
        Polygon::new(vec![
            Point::new(50, 9),
            Point::new(40, 2),
            Point::new(45, 1),
        ]),
        rectangle(0, 0, 5, 10),
    ];
    let expected = [
        snapshot_polygons(&total_result),
        snapshot_polygons(&limiting_result),
    ]
    .concat();
    let mut total_result = Some(total_result);
    let mut limiting_result = Some(limiting_result);
    let calls = std::cell::Cell::new(0);

    let output = prepare_candidate_boundary_polylines_using(
        operation_input(&area, &total, 7, 11.0),
        |subject, _| {
            let call = calls.get();
            calls.set(call + 1);
            match call {
                0 => {
                    assert_eq!(subject.as_ptr(), total.as_ptr());
                    Ok(total_result.take().unwrap())
                }
                1 => {
                    assert_eq!(subject.as_ptr(), area.limiting_area.as_ptr());
                    Ok(limiting_result.take().unwrap())
                }
                _ => unreachable!(),
            }
        },
    )
    .unwrap()
    .unwrap();

    assert_eq!(calls.get(), 2);
    assert_eq!(
        snapshot_polylines(&output),
        expected
            .into_iter()
            .map(|mut points| {
                points.push(points[0]);
                points
            })
            .collect::<Vec<_>>()
    );
}

#[test]
fn task22o59_each_offset_polygon_gets_exactly_one_closure_duplicate() {
    let area = candidate_area(vec![rectangle(0, 0, 1, 1)], Vec::new());
    let empty = Polygon::new(Vec::new());
    let result = std::panic::catch_unwind(|| {
        prepare_candidate_boundary_polylines_using(operation_input(&area, &[], 1, 1.0), |_, _| {
            Ok(vec![empty.clone()])
        })
    });

    assert!(
        result.is_err(),
        "source-valid offset polygons must be nonempty"
    );
}
