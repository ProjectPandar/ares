use super::*;

#[test]
fn task22o65_distinguishes_all_absent_and_present_empty_key_combinations() {
    assert_eq!(
        collect_bridge_rewrite_areas(None, None, CoordinateScale::Normal).unwrap(),
        None
    );
    for (current, upper) in [
        (Some([].as_slice()), None),
        (None, Some([].as_slice())),
        (Some([].as_slice()), Some([].as_slice())),
    ] {
        assert_eq!(
            collect_bridge_rewrite_areas(current, upper, CoordinateScale::Normal).unwrap(),
            Some(BridgeRewriteAreas {
                cut_from_infill: Vec::new(),
                additional_ensuring_areas: Vec::new(),
            })
        );
    }
}

#[test]
fn task22o65_current_only_clones_flat_order_without_geometry_or_validation() {
    let high = i64::MAX;
    let current = vec![
        candidate(
            4,
            vec![
                polygon(&[(high, 0), (high - 1, 1), (high - 2, 0)]),
                rectangle(70, 0, 80, 10),
            ],
        ),
        candidate(1, vec![rectangle(-20, 0, -10, 10)]),
    ];
    let before = candidate_snapshot(&current);
    let input_ptrs = current
        .iter()
        .flat_map(|candidate| candidate.new_polygons.iter().map(Polygon::points))
        .map(<[_]>::as_ptr)
        .collect::<Vec<_>>();

    let output = collect_bridge_rewrite_areas(Some(&current), None, CoordinateScale::Normal)
        .unwrap()
        .unwrap();

    assert_eq!(snapshot(&output.cut_from_infill), before.concat());
    assert_eq!(candidate_snapshot(&current), before);
    assert!(
        output
            .cut_from_infill
            .iter()
            .zip(input_ptrs)
            .all(|(output, input_ptr)| output.points().as_ptr() != input_ptr)
    );
    assert!(output.additional_ensuring_areas.is_empty());
}
