use super::fixture::{input, options, square, surface};
use crate::{
    ProcessEnsureVerticalShellThickness, SliceError,
    geometry::{CoordinateScale, JoinType, Point, Polygon, offset_paths},
    project_slice::{
        prepare_infill::horizontal_shell_propagation::{
            GeometryStep, fail_geometry_at, fail_geometry_at_occurrence, geometry, geometry_events,
            reset_hooks, types::NeighborOutcome,
        },
        region_slices::RegionSurfaceKind,
    },
};

macro_rules! context {
    ($source:expr, $neighbor:expr, $options:expr, $fill:expr) => {
        geometry::NeighborContext {
            scale: CoordinateScale::Normal,
            source_input: $source,
            neighbor_input: $neighbor,
            options: $options,
            neighbor_fill: $fill,
        }
    };
}

fn internal(size: i64) -> Vec<crate::project_slice::region_slices::RegionSurface> {
    vec![surface(RegionSurfaceKind::Internal, square(0, 0, size))]
}

#[test]
fn acute_repair_growth_is_miter3_not_adjacent_opening_miter5() {
    let paths = vec![Polygon::new(vec![
        Point::new(0, 0),
        Point::new(268, 1_000),
        Point::new(-268, 1_000),
    ])];
    let actual = geometry::repair_expansion_for_test(&paths, 100.0).unwrap();
    let miter3 = offset_paths(&paths, 100.0, JoinType::Miter, 3.0).unwrap();
    let miter5 = offset_paths(&paths, 100.0, JoinType::Miter, 5.0).unwrap();
    assert_eq!(actual, miter3);
    assert_ne!(actual, miter5);
}

#[test]
fn scaled_flow_width_preserves_truncating_integer_to_f32_order_for_both_scales() {
    let width = 0.123_456_7_f32;
    assert_eq!(
        geometry::scaled_width(CoordinateScale::Normal, width)
            .unwrap()
            .to_bits(),
        ((f64::from(width) / CoordinateScale::Normal.factor()) as i64 as f32).to_bits()
    );
    assert_eq!(
        geometry::scaled_width(CoordinateScale::LargeBed, width)
            .unwrap()
            .to_bits(),
        ((f64::from(width) / CoordinateScale::LargeBed.factor()) as i64 as f32).to_bits()
    );
}

#[test]
fn density_and_mode_factors_are_exact_and_exhaustive() {
    let mut options = options();
    options.sparse_infill_density.0 = 15.0;
    for (mode, first, second) in [
        (ProcessEnsureVerticalShellThickness::None, 0.5, 1.0),
        (ProcessEnsureVerticalShellThickness::CriticalOnly, 0.2, 3.0),
        (ProcessEnsureVerticalShellThickness::Moderate, 0.0, 3.0),
    ] {
        options.ensure_vertical_shell_thickness = mode;
        assert_eq!(geometry::first_factor(&options), first);
        assert_eq!(geometry::second_factor(&options), second);
    }
    options.sparse_infill_density.0 = 0.0;
    for mode in [
        ProcessEnsureVerticalShellThickness::None,
        ProcessEnsureVerticalShellThickness::CriticalOnly,
        ProcessEnsureVerticalShellThickness::Moderate,
    ] {
        options.ensure_vertical_shell_thickness = mode;
        assert_eq!(geometry::first_factor(&options), 1.0);
    }
}

#[test]
fn moderate_skips_first_filter_and_second_filter_does_not_replace_carried_solid() {
    let source = input(0.2, 0.2);
    let neighbor = input(0.2, 0.2);
    let mut options = options();
    options.ensure_vertical_shell_thickness = ProcessEnsureVerticalShellThickness::Moderate;
    options.sparse_infill_density.0 = 15.0;
    let fill = internal(500_000);
    let original = vec![square(0, 0, 500_000).contour().clone()];
    let mut solid = original.clone();

    reset_hooks();
    let outcome = geometry::process_neighbor(
        context!(&source, Some(&neighbor), &options, &fill),
        &mut solid,
    )
    .unwrap();
    assert!(matches!(outcome, NeighborOutcome::Rebuilt(_)));
    assert_eq!(solid, original);
    assert_eq!(
        &geometry_events()[..5],
        &[
            GeometryStep::SafetyIntersection,
            GeometryStep::SourceSolidWidthScale,
            GeometryStep::SecondOpeningShrink,
            GeometryStep::SecondOpeningExpand,
            GeometryStep::SecondTooNarrowDifference,
        ]
    );
}

#[test]
fn none_mode_uses_neighbor_external_width_and_updates_carried_solid_in_first_filter() {
    let source = input(0.01, 0.01);
    let neighbor = input(0.2, 0.01);
    let mut options = options();
    options.ensure_vertical_shell_thickness = ProcessEnsureVerticalShellThickness::None;
    options.sparse_infill_density.0 = 15.0;
    let fill = internal(100_000);
    let mut solid = vec![square(0, 0, 100_000).contour().clone()];

    reset_hooks();
    let outcome = geometry::process_neighbor(
        context!(&source, Some(&neighbor), &options, &fill),
        &mut solid,
    )
    .unwrap();
    assert!(matches!(outcome, NeighborOutcome::Rebuilt(_)));
    assert!(solid.is_empty());
    assert!(geometry_events().contains(&GeometryStep::FirstTrimDifference));
}

#[test]
fn empty_neighbor_runs_safety_intersection_before_mode_specific_stop_decision() {
    let source = input(0.2, 0.2);
    let mut options = options();
    let mut solid = vec![square(0, 0, 100).contour().clone()];
    reset_hooks();
    assert!(matches!(
        geometry::process_neighbor(context!(&source, None, &options, &[]), &mut solid).unwrap(),
        NeighborOutcome::EmptyIntersection
    ));
    assert_eq!(geometry_events(), vec![GeometryStep::SafetyIntersection]);
    options.ensure_vertical_shell_thickness = ProcessEnsureVerticalShellThickness::Moderate;
    options.sparse_infill_density.0 = 15.0;
    assert!(!geometry::should_stop_after_empty(&options));
    options.ensure_vertical_shell_thickness = ProcessEnsureVerticalShellThickness::CriticalOnly;
    assert!(geometry::should_stop_after_empty(&options));
    options.ensure_vertical_shell_thickness = ProcessEnsureVerticalShellThickness::None;
    assert!(geometry::should_stop_after_empty(&options));
    options.ensure_vertical_shell_thickness = ProcessEnsureVerticalShellThickness::Moderate;
    options.sparse_infill_density.0 = 0.0;
    assert!(geometry::should_stop_after_empty(&options));
}

#[test]
fn every_ordered_geometry_site_maps_injected_failure_to_the_stable_error() {
    let first_steps = [
        GeometryStep::SafetyIntersection,
        GeometryStep::NeighborExternalWidthScale,
        GeometryStep::FirstOpeningShrink,
        GeometryStep::FirstOpeningExpand,
        GeometryStep::FirstTooNarrowDifference,
        GeometryStep::FirstTrimDifference,
    ];
    let later_steps = [
        GeometryStep::SourceSolidWidthScale,
        GeometryStep::SecondOpeningShrink,
        GeometryStep::SecondOpeningExpand,
        GeometryStep::SecondTooNarrowDifference,
        GeometryStep::RepairExpansion,
        GeometryStep::RepairIntersection,
        GeometryStep::SolidUnion,
        GeometryStep::InternalSafetyDifference,
        GeometryStep::ExternalGroupDifference,
    ];
    for (steps, mode, size) in [
        (
            first_steps.as_slice(),
            ProcessEnsureVerticalShellThickness::None,
            100_000,
        ),
        (
            later_steps.as_slice(),
            ProcessEnsureVerticalShellThickness::Moderate,
            500_000,
        ),
    ] {
        for &step in steps {
            let source = input(0.2, 0.2);
            let neighbor = input(0.2, 0.2);
            let mut options = options();
            options.ensure_vertical_shell_thickness = mode;
            options.sparse_infill_density.0 = 15.0;
            let mut fill = internal(size);
            fill.push(surface(
                RegionSurfaceKind::Top,
                square(size + 1_000, 0, 500),
            ));
            let mut solid = vec![square(0, 0, size).contour().clone()];
            reset_hooks();
            fail_geometry_at(step);
            let Err(error) = geometry::process_neighbor(
                context!(&source, Some(&neighbor), &options, &fill),
                &mut solid,
            ) else {
                panic!("selected geometry step {step:?} must be reached");
            };
            assert_eq!(
                error,
                SliceError::InvalidInput(
                    "horizontal-shell propagation geometry is outside the supported Clipper range"
                        .to_owned()
                )
            );
        }
    }
}

#[test]
fn occurrence_failure_hook_targets_the_selected_ordered_geometry_visit() {
    let source = input(0.2, 0.2);
    let options = options();
    let mut solid = vec![square(0, 0, 100).contour().clone()];
    reset_hooks();
    fail_geometry_at_occurrence(GeometryStep::SafetyIntersection, 2);
    assert!(matches!(
        geometry::process_neighbor(context!(&source, None, &options, &[]), &mut solid),
        Ok(NeighborOutcome::EmptyIntersection)
    ));
    let Err(error) = geometry::process_neighbor(context!(&source, None, &options, &[]), &mut solid)
    else {
        panic!("the second selected geometry visit must fail");
    };
    assert_eq!(
        error,
        SliceError::InvalidInput(
            "horizontal-shell propagation geometry is outside the supported Clipper range"
                .to_owned()
        )
    );
}

#[test]
fn aligned_flow_scaling_failures_use_the_single_stable_error_in_order() {
    let mut options = options();
    options.ensure_vertical_shell_thickness = ProcessEnsureVerticalShellThickness::None;
    options.sparse_infill_density.0 = 15.0;
    let source = input(0.2, 0.2);
    let invalid_neighbor = input(f32::MAX, 0.2);
    let fill = internal(1_000_000);
    let mut solid = vec![square(0, 0, 1_000_000).contour().clone()];
    reset_hooks();
    let Err(error) = geometry::process_neighbor(
        context!(&source, Some(&invalid_neighbor), &options, &fill),
        &mut solid,
    ) else {
        panic!("invalid aligned flow must fail before opening geometry");
    };
    assert_eq!(
        error,
        SliceError::InvalidInput(
            "horizontal-shell propagation geometry is outside the supported Clipper range"
                .to_owned()
        )
    );
    assert_eq!(
        geometry_events(),
        vec![
            GeometryStep::SafetyIntersection,
            GeometryStep::NeighborExternalWidthScale,
        ]
    );

    options.ensure_vertical_shell_thickness = ProcessEnsureVerticalShellThickness::Moderate;
    let invalid_source = input(0.2, f32::MAX);
    let neighbor = input(0.2, 0.2);
    let mut solid = vec![square(0, 0, 1_000_000).contour().clone()];
    reset_hooks();
    let Err(error) = geometry::process_neighbor(
        context!(&invalid_source, Some(&neighbor), &options, &fill),
        &mut solid,
    ) else {
        panic!("invalid source solid flow must fail before the second opening");
    };
    assert_eq!(error, super::super::range_error());
    assert_eq!(
        geometry_events(),
        vec![
            GeometryStep::SafetyIntersection,
            GeometryStep::SourceSolidWidthScale,
        ]
    );
}
