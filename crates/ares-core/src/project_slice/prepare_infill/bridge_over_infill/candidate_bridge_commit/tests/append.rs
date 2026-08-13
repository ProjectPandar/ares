use super::*;

#[test]
fn task22o64_appends_every_call_in_order_and_moves_owned_allocations() {
    let mut completed = Vec::with_capacity(3);
    let cases = [
        (source(7, 3, 11), 70, 0.37_f64.to_bits(), -70),
        (source(7, 1, 4), -20, 0x7ff8_0000_0000_0123_u64, 20),
        (source(7, 8, 2), 40, (-1.25_f64).to_bits(), -40),
    ];

    for (expected_index, (source, bridge_x, angle_bits, expansion_x)) in
        cases.into_iter().enumerate()
    {
        let bridging_area = vec![rectangle(bridge_x, 0, bridge_x + 10, 10)];
        let bridge_ptr = bridging_area.as_ptr();
        let expansion = vec![rectangle(expansion_x, -20, expansion_x + 10, -10)];
        let expansion_ptr = expansion.as_ptr();

        let returned = append_postprocessed_candidate(
            &mut completed,
            source,
            postprocessed(bridging_area, f64::from_bits(angle_bits), expansion),
        );

        assert_eq!(completed.len(), expected_index + 1);
        assert_eq!(completed[expected_index].source, source);
        assert_eq!(completed[expected_index].bridge_angle.to_bits(), angle_bits);
        assert_eq!(completed[expected_index].new_polygons.as_ptr(), bridge_ptr);
        assert_eq!(returned.as_ptr(), expansion_ptr);
    }

    assert_eq!(
        completed
            .iter()
            .map(|candidate| candidate.source.surface_index)
            .collect::<Vec<_>>(),
        vec![11, 4, 2]
    );
}

#[test]
fn task22o64_empty_final_bridge_still_appends_and_returns_expansion() {
    let mut completed = vec![candidate(source(9, 0, 0), 50, 0.5)];
    let expansion = vec![rectangle(-50, -50, 50, 50)];
    let expansion_ptr = expansion.as_ptr();
    let angle_bits = 0x8000_0000_0000_0000_u64;

    let returned = append_postprocessed_candidate(
        &mut completed,
        source(9, 2, 6),
        postprocessed(Vec::new(), f64::from_bits(angle_bits), expansion),
    );

    assert_eq!(completed.len(), 2);
    assert_eq!(completed[1].source, source(9, 2, 6));
    assert!(completed[1].new_polygons.is_empty());
    assert_eq!(completed[1].bridge_angle.to_bits(), angle_bits);
    assert_eq!(returned.as_ptr(), expansion_ptr);
}

#[test]
fn task22o64_independent_owned_inputs_are_repeatable() {
    let run = || {
        let mut completed = Vec::new();
        let expansion = append_postprocessed_candidate(
            &mut completed,
            source(3, 5, 7),
            postprocessed(
                vec![rectangle(30, 0, 40, 10)],
                1.75,
                vec![rectangle(-30, -30, 30, 30)],
            ),
        );
        (snapshot(&completed), expansion)
    };

    let first = run();
    let second = run();
    assert_eq!(first, second);
}
