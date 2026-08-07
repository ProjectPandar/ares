use super::*;
use crate::geometry::clipper::{ClipOperation, FillRule};

#[test]
fn clear_removes_z_inputs_and_collector_state() {
    let mut clipper = crossing_clipper();
    let (_, first_pairs) = clipper.execute_z_paths(
        ClipOperation::Intersection,
        FillRule::NonZero,
        FillRule::NonZero,
    );
    assert_eq!(first_pairs, vec![(1, 2), (1, 2)]);
    clipper.clear();
    assert_eq!(clipper.z_state_for_test(), (0, true));
    clipper
        .add_z_closed_path(
            &path(&[(0, 0), (10, 0), (10, 10), (0, 10)], 4),
            PathRole::Clip,
        )
        .unwrap();
    clipper
        .add_z_open_path(&path(&[(-5, 5), (15, 5)], 9), PathRole::Subject)
        .unwrap();
    let (_, second_pairs) = clipper.execute_z_paths(
        ClipOperation::Intersection,
        FillRule::NonZero,
        FillRule::NonZero,
    );
    assert_eq!(second_pairs, vec![(4, 9), (4, 9)]);
}

#[cfg(debug_assertions)]
#[test]
fn z_execution_rejects_an_active_prior_collector() {
    let mut clipper = Clipper::new(ClipperOptions::default());
    clipper.activate_z_collector_for_test();
    assert!(
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            clipper.execute_z_paths(ClipOperation::Union, FillRule::NonZero, FillRule::NonZero)
        }))
        .is_err()
    );
}

#[cfg(not(debug_assertions))]
pub(crate) mod release {
    use crate::geometry::clipper::z::z_fill_for_test;

    #[test]
    fn collector_uses_first_two_sorted_labels() {
        assert_eq!(z_fill_for_test([9, 2, 7, 4]), (-1, vec![(2, 4)]));
    }
}
