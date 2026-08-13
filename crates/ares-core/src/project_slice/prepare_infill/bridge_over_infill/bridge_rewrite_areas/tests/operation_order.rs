use std::cell::Cell;

use super::*;

#[test]
fn task22o65_uses_each_flow_and_runs_whole_set_shrink_then_difference_in_order() {
    for (scale, expected_delta) in [
        (CoordinateScale::Normal, -167_772_176.0_f32),
        (CoordinateScale::LargeBed, -16_777_216.0_f32),
    ] {
        let surfaces = [
            candidate(
                9,
                vec![rectangle(70, 0, 80, 10), rectangle(-70, 0, -60, 10)],
            ),
            candidate(2, vec![rectangle(20, 0, 30, 10)]),
        ];
        let upper = [
            UpperBridgeEnsuringInput {
                surface: &surfaces[0],
                solid_infill_flow: flow(167.772_17),
            },
            UpperBridgeEnsuringInput {
                surface: &surfaces[1],
                solid_infill_flow: flow(0.45),
            },
        ];
        let step = Cell::new(0);
        let first_shrunk = vec![rectangle(60, 1, 61, 2)];
        let second_shrunk = vec![rectangle(10, 1, 11, 2)];
        let first_ring = vec![rectangle(90, 0, 91, 1), rectangle(-90, 0, -89, 1)];
        let second_ring = vec![rectangle(40, 0, 41, 1)];

        let output = collect_bridge_rewrite_areas_using(
            None,
            Some(&upper),
            scale,
            |subject, delta| {
                let current = step.get();
                step.set(current + 1);
                match current {
                    0 => {
                        assert_eq!(subject.as_ptr(), surfaces[0].new_polygons.as_ptr());
                        assert_eq!(subject.len(), 2);
                        assert_eq!(delta.to_bits(), expected_delta.to_bits());
                        Ok(first_shrunk.clone())
                    }
                    2 => {
                        assert_eq!(subject.as_ptr(), surfaces[1].new_polygons.as_ptr());
                        let expected = match scale {
                            CoordinateScale::Normal => -449_999.0_f32,
                            CoordinateScale::LargeBed => -44_999.0_f32,
                        };
                        assert_eq!(delta.to_bits(), expected.to_bits());
                        Ok(second_shrunk.clone())
                    }
                    _ => panic!("shrink must alternate with difference"),
                }
            },
            |subject, clip| {
                let current = step.get();
                step.set(current + 1);
                match current {
                    1 => {
                        assert_eq!(subject.as_ptr(), surfaces[0].new_polygons.as_ptr());
                        assert_eq!(snapshot(clip), snapshot(&first_shrunk));
                        Ok(first_ring.clone())
                    }
                    3 => {
                        assert_eq!(subject.as_ptr(), surfaces[1].new_polygons.as_ptr());
                        assert_eq!(snapshot(clip), snapshot(&second_shrunk));
                        Ok(second_ring.clone())
                    }
                    _ => panic!("difference must follow its candidate shrink"),
                }
            },
        )
        .unwrap()
        .unwrap();

        assert_eq!(step.get(), 4);
        assert_eq!(
            snapshot(&output.additional_ensuring_areas),
            snapshot(&[first_ring, second_ring].concat())
        );
    }
}

#[test]
fn task22o65_empty_upper_candidate_still_runs_shrink_then_difference() {
    let surface = candidate(6, Vec::new());
    let upper = [UpperBridgeEnsuringInput {
        surface: &surface,
        solid_infill_flow: flow(0.000_01),
    }];
    let step = Cell::new(0);

    let output = collect_bridge_rewrite_areas_using(
        None,
        Some(&upper),
        CoordinateScale::Normal,
        |subject, delta| {
            assert!(subject.is_empty());
            assert_eq!(delta.to_bits(), (-9.0_f32).to_bits());
            assert_eq!(step.replace(1), 0);
            Ok(Vec::new())
        },
        |subject, clip| {
            assert!(subject.is_empty());
            assert!(clip.is_empty());
            assert_eq!(step.replace(2), 1);
            Ok(Vec::new())
        },
    )
    .unwrap()
    .unwrap();

    assert_eq!(step.get(), 2);
    assert!(output.cut_from_infill.is_empty());
    assert!(output.additional_ensuring_areas.is_empty());
    assert!(surface.new_polygons.is_empty());
}

#[test]
fn task22o65_injected_errors_stop_without_reaching_later_candidates() {
    let surfaces = [
        candidate(0, vec![rectangle(0, 0, 100, 100)]),
        candidate(1, vec![rectangle(200, 0, 300, 100)]),
    ];
    let upper = [
        UpperBridgeEnsuringInput {
            surface: &surfaces[0],
            solid_infill_flow: flow(0.000_01),
        },
        UpperBridgeEnsuringInput {
            surface: &surfaces[1],
            solid_infill_flow: flow(0.000_01),
        },
    ];

    for fail_at in 0..4 {
        let step = Cell::new(0);
        let visit = || {
            let current = step.get();
            step.set(current + 1);
            if current == fail_at {
                Err(ClipperError::CoordinateOutOfRange)
            } else {
                Ok(Vec::new())
            }
        };
        let result = collect_bridge_rewrite_areas_using(
            None,
            Some(&upper),
            CoordinateScale::Normal,
            |_, _| visit(),
            |_, _| visit(),
        );
        assert_eq!(result.unwrap_err(), ClipperError::CoordinateOutOfRange);
        assert_eq!(step.get(), fail_at + 1);
    }
}
