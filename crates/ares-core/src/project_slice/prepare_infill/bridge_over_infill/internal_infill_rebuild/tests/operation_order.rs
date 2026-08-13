use std::cell::Cell;

use super::*;

#[test]
fn task22o67_filters_internal_in_source_order_then_runs_two_differences() {
    let surfaces = [
        surface(
            RegionSurfaceKind::Top,
            expolygon(rectangle(-100, 0, -50, 50), Vec::new()),
        ),
        surface(
            RegionSurfaceKind::Internal,
            expolygon(rectangle(0, 0, 100, 100), vec![rectangle(20, 20, 40, 40)]),
        ),
        surface(
            RegionSurfaceKind::InternalSolid,
            expolygon(rectangle(150, 0, 200, 50), Vec::new()),
        ),
        surface(
            RegionSurfaceKind::Internal,
            expolygon(rectangle(300, 0, 400, 100), Vec::new()),
        ),
    ];
    let cut = [rectangle(50, -10, 60, 110)];
    let ensuring = [expolygon(rectangle(310, 0, 320, 100), Vec::new())];
    let expected_internal = vec![
        expolygon(rectangle(0, 0, 100, 100), vec![rectangle(20, 20, 40, 40)]),
        expolygon(rectangle(300, 0, 400, 100), Vec::new()),
    ];
    let after_cut = vec![expolygon(rectangle(0, 0, 50, 100), Vec::new())];
    let final_output = vec![
        expolygon(rectangle(320, 0, 400, 100), Vec::new()),
        expolygon(rectangle(0, 0, 50, 100), Vec::new()),
    ];
    let step = Cell::new(0);

    let output = rebuild_internal_infills_using(
        &surfaces,
        &cut,
        &ensuring,
        |subject, clip| {
            assert_eq!(step.replace(1), 0);
            assert_eq!(snapshot_ex(subject), snapshot_ex(&expected_internal));
            assert_eq!(clip.as_ptr(), cut.as_ptr());
            Ok(after_cut.clone())
        },
        |subject, clip| {
            assert_eq!(step.replace(2), 1);
            assert_eq!(snapshot_ex(subject), snapshot_ex(&after_cut));
            assert_eq!(clip.as_ptr(), ensuring.as_ptr());
            Ok(final_output.clone())
        },
    )
    .unwrap();

    assert_eq!(step.get(), 2);
    assert_eq!(
        surface_snapshot(&output)
            .iter()
            .map(|entry| entry.0)
            .collect::<Vec<_>>(),
        vec![RegionSurfaceKind::Internal; 2]
    );
    assert_eq!(
        surface_snapshot(&output)
            .iter()
            .map(|entry| (entry.2, entry.3, entry.4, entry.5))
            .collect::<Vec<_>>(),
        vec![((-1.0_f64).to_bits(), 1, (-1.0_f64).to_bits(), 0); 2]
    );
    assert_eq!(
        output
            .iter()
            .map(|surface| surface.as_parts().1.clone())
            .collect::<Vec<_>>(),
        final_output
    );
}

#[test]
fn task22o67_empty_inputs_still_run_both_operations() {
    let step = Cell::new(0);
    let output = rebuild_internal_infills_using(
        &[],
        &[],
        &[],
        |subject, clip| {
            assert_eq!(step.replace(1), 0);
            assert!(subject.is_empty() && clip.is_empty());
            Ok(Vec::new())
        },
        |subject, clip| {
            assert_eq!(step.replace(2), 1);
            assert!(subject.is_empty() && clip.is_empty());
            Ok(Vec::new())
        },
    )
    .unwrap();
    assert!(output.is_empty());
    assert_eq!(step.get(), 2);
}

#[test]
fn task22o67_first_error_stops_before_later_work() {
    for fail_at in 0..2 {
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
        let result = rebuild_internal_infills_using(&[], &[], &[], |_, _| visit(), |_, _| visit());
        assert_eq!(result.err().unwrap(), ClipperError::CoordinateOutOfRange);
        assert_eq!(step.get(), fail_at + 1);
    }
}
