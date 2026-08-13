use super::*;
use std::cell::{Cell, RefCell};

#[test]
fn task22o58_preserves_roles_predicate_order_original_survivors_and_union_input() {
    let candidates = [rectangle(0, 0, 10, 10), rectangle(20, 0, 30, 10)];
    let deep = [rectangle(-10, -10, 40, 20)];
    let unsupported = [rectangle(0, 0, 40, 20)];
    let expansion = [rectangle(100, 0, 110, 10)];
    let expanded = vec![rectangle(-1, -1, 11, 11)];
    let filtered = vec![
        rectangle(1, 0, 5, 5),
        rectangle(10, 0, 15, 5),
        rectangle(20, 0, 25, 5),
    ];
    let survivor_ptrs = [
        filtered[0].points().as_ptr() as usize,
        filtered[2].points().as_ptr() as usize,
    ];
    let mut expanded = Some(expanded);
    let mut filtered = Some(filtered);
    let call = Cell::new(0);
    let union_calls = Cell::new(0);
    let spacing = 16_777_217_i64;

    let output = prepare_candidate_bridge_area_using(
        operation_input(&candidates, &deep, &unsupported, &expansion, spacing),
        |subject, delta| {
            assert_eq!(subject.as_ptr(), candidates.as_ptr());
            assert_eq!(delta.to_bits(), (spacing as f32).to_bits());
            Ok(expanded.take().unwrap())
        },
        |subject, clip| {
            let index = call.get();
            call.set(index + 1);
            match index {
                0 => {
                    assert_eq!(
                        snapshot(subject),
                        vec![vec![(-1, -1), (11, -1), (11, 11), (-1, 11)]]
                    );
                    assert_eq!(clip.as_ptr(), deep.as_ptr());
                    Ok(filtered.take().unwrap())
                }
                1..=3 => {
                    assert_eq!(subject.len(), 1);
                    assert_eq!(clip.as_ptr(), unsupported.as_ptr());
                    if index == 2 {
                        Ok(Vec::new())
                    } else {
                        Ok(vec![subject[0].clone()])
                    }
                }
                _ => unreachable!(),
            }
        },
        |union_input| {
            union_calls.set(union_calls.get() + 1);
            assert_eq!(union_input.len(), 3);
            assert_eq!(
                snapshot(&union_input[..2]),
                vec![
                    vec![(1, 0), (5, 0), (5, 5), (1, 5)],
                    vec![(20, 0), (25, 0), (25, 5), (20, 5)],
                ]
            );
            assert_ne!(union_input[0].points().as_ptr() as usize, survivor_ptrs[0]);
            assert_ne!(union_input[1].points().as_ptr() as usize, survivor_ptrs[1]);
            assert_eq!(snapshot(&union_input[2..]), snapshot(&expansion));
            Ok(vec![rectangle(0, 0, 110, 10)])
        },
    )
    .unwrap();

    assert_eq!(call.get(), 4);
    assert_eq!(union_calls.get(), 1);
    assert_eq!(
        output
            .area_to_be_bridge
            .iter()
            .map(|polygon| polygon.points().as_ptr() as usize)
            .collect::<Vec<_>>(),
        survivor_ptrs
    );
    assert_eq!(
        snapshot(&output.limiting_area),
        vec![vec![(0, 0), (110, 0), (110, 10), (0, 10)]]
    );
}

#[test]
fn task22o58_injected_later_predicate_error_short_circuits_before_union() {
    let candidates = [rectangle(0, 0, 10, 10)];
    let deep = [rectangle(0, 0, 20, 20)];
    let unsupported = [rectangle(0, 0, 20, 20)];
    let calls = Cell::new(0);
    let result = prepare_candidate_bridge_area_using(
        operation_input(&candidates, &deep, &unsupported, &[], 1),
        |subject, _| Ok(subject.to_vec()),
        |subject, _| {
            let index = calls.get();
            calls.set(index + 1);
            match index {
                0 => Ok(vec![subject[0].clone(), subject[0].clone()]),
                1 => Ok(vec![subject[0].clone()]),
                2 => Err(ClipperError::CoordinateOutOfRange),
                _ => unreachable!(),
            }
        },
        |_| panic!("union must not run after a predicate error"),
    );
    assert_range_error(result);
    assert_eq!(calls.get(), 3);
}

#[test]
fn task22o58_injected_competing_errors_preserve_every_stage_ordinal() {
    let candidates = [rectangle(0, 0, 10, 10)];
    let deep = [rectangle(-10, -10, 20, 20)];
    let unsupported = [rectangle(-10, -10, 20, 20)];
    let expected = [
        vec!["offset"],
        vec!["offset", "deep"],
        vec!["offset", "deep", "predicate"],
        vec!["offset", "deep", "predicate", "union"],
    ];

    for (fail_at, expected_trace) in expected.iter().enumerate() {
        let trace = RefCell::new(Vec::new());
        let intersection_calls = Cell::new(0);
        let result = prepare_candidate_bridge_area_using(
            operation_input(&candidates, &deep, &unsupported, &[], 1),
            |subject, _| {
                trace.borrow_mut().push("offset");
                if fail_at == 0 {
                    Err(ClipperError::CoordinateOutOfRange)
                } else {
                    Ok(subject.to_vec())
                }
            },
            |subject, _| {
                let call = intersection_calls.get();
                intersection_calls.set(call + 1);
                let stage = if call == 0 { "deep" } else { "predicate" };
                trace.borrow_mut().push(stage);
                if fail_at == call + 1 {
                    Err(ClipperError::CoordinateOutOfRange)
                } else {
                    Ok(vec![subject[0].clone()])
                }
            },
            |subject| {
                trace.borrow_mut().push("union");
                if fail_at == 3 {
                    Err(ClipperError::CoordinateOutOfRange)
                } else {
                    Ok(subject.to_vec())
                }
            },
        );

        assert_range_error(result);
        assert_eq!(&*trace.borrow(), expected_trace);
    }
}
