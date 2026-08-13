use super::*;

#[test]
fn task22o58_natural_errors_preserve_engine_call_precedence() {
    let valid = [rectangle(0, 0, 100, 100)];
    let broad = [rectangle(-100, -100, 200, 200)];
    let invalid = [outside_range()];

    assert_range_error(prepare_candidate_bridge_area(
        &invalid,
        &broad,
        &broad,
        &[],
        10,
    ));
    assert_range_error(prepare_candidate_bridge_area(
        &valid,
        &invalid,
        &broad,
        &[],
        10,
    ));
    assert_range_error(prepare_candidate_bridge_area(
        &valid,
        &broad,
        &invalid,
        &[],
        10,
    ));
    assert_range_error(prepare_candidate_bridge_area(&[], &[], &[], &invalid, 10));
}

#[test]
fn task22o58_errors_are_atomic_and_all_borrowed_allocations_are_unchanged() {
    let candidates = vec![rectangle(0, 0, 100, 100)];
    let deep = vec![rectangle(-100, -100, 200, 200)];
    let unsupported = vec![outside_range()];
    let expansion = vec![rectangle(300, 0, 400, 100)];
    let before = snapshot_inputs(&candidates, &deep, &unsupported, &expansion);

    assert_range_error(prepare_candidate_bridge_area(
        &candidates,
        &deep,
        &unsupported,
        &expansion,
        10,
    ));
    assert_eq!(
        snapshot_inputs(&candidates, &deep, &unsupported, &expansion),
        before
    );
}

#[test]
fn task22o58_repeat_calls_are_identical_and_preserve_inputs() {
    let candidates = vec![
        rectangle(0, 0, 1_000, 1_000),
        rectangle(2_000, 0, 3_000, 1_000),
    ];
    let deep = vec![rectangle(-100, -100, 3_100, 1_100)];
    let unsupported = vec![rectangle(-100, -100, 3_100, 1_100)];
    let expansion = vec![rectangle(1_000, 0, 2_000, 1_000)];
    let before = snapshot_inputs(&candidates, &deep, &unsupported, &expansion);

    let first = prepare(&candidates, &deep, &unsupported, &expansion, 100);
    let second = prepare(&candidates, &deep, &unsupported, &expansion, 100);
    assert_eq!(
        snapshot(&first.area_to_be_bridge),
        snapshot(&second.area_to_be_bridge)
    );
    assert_eq!(
        snapshot(&first.limiting_area),
        snapshot(&second.limiting_area)
    );
    assert_eq!(
        snapshot_inputs(&candidates, &deep, &unsupported, &expansion),
        before
    );
}

#[derive(Debug, Eq, PartialEq)]
struct InputsSnapshot {
    outer: [(usize, usize, usize); 4],
    paths: Vec<(usize, Vec<(i64, i64)>)>,
}

fn snapshot_inputs(
    candidates: &Vec<Polygon>,
    deep: &Vec<Polygon>,
    unsupported: &Vec<Polygon>,
    expansion: &Vec<Polygon>,
) -> InputsSnapshot {
    let vectors = [candidates, deep, unsupported, expansion];
    InputsSnapshot {
        outer: vectors.map(|paths| (paths.as_ptr() as usize, paths.len(), paths.capacity())),
        paths: vectors
            .into_iter()
            .flat_map(|paths| paths.iter())
            .map(|polygon| {
                (
                    polygon.points().as_ptr() as usize,
                    snapshot(std::slice::from_ref(polygon)).remove(0),
                )
            })
            .collect(),
    }
}
