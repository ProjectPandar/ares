use crate::{
    ProcessEnsureVerticalShellThickness,
    project_slice::prepare_infill::vertical_shell_projection::{
        GeometryStep, gather, geometry_events, reset_geometry_hooks,
    },
};

use super::{cache, layer, lslice, options, projection_input};

#[test]
fn task22o20_count_windows_visit_top_then_bottom() {
    reset_geometry_hooks();
    let caches = vec![cache(0), cache(0), cache(0), cache(0)];
    let layers = vec![
        layer(0, 0.2, 0.2),
        layer(1, 0.2, 0.4),
        layer(2, 0.2, 0.6),
        layer(3, 0.2, 0.8),
    ];
    let lslices = vec![lslice(-100, 100); 4];
    let mut options = options();
    options.top_shell_layers.0 = 2;
    options.top_shell_thickness.0 = 0.0;
    options.bottom_shell_layers.0 = 2;
    options.bottom_shell_thickness.0 = 0.0;
    let output =
        gather::project_record(1, projection_input(&caches, &layers, &lslices, &options, 5))
            .unwrap();
    assert!(!output.shell.is_empty());
    assert_eq!(
        geometry_events(),
        [
            GeometryStep::TopVisit,
            GeometryStep::HoleIntersection,
            GeometryStep::BottomVisit,
            GeometryStep::HoleIntersection,
            GeometryStep::ShellUnion,
        ]
    );
}

#[test]
fn task22o20_thickness_equality_is_excluded_and_one_step_below_is_included() {
    let caches = vec![cache(0), cache(0), cache(0)];
    let lslices = vec![lslice(-100, 100); 3];
    let mut options = options();
    options.top_shell_layers.0 = 1;
    options.bottom_shell_layers.0 = 0;
    options.top_shell_thickness.0 = 0.2001;

    let boundary = options.top_shell_thickness.0 - 1e-4;
    reset_geometry_hooks();
    let equality = vec![
        layer(0, 0.2, 0.0),
        layer(1, 0.2, boundary),
        layer(2, 0.2, 0.6),
    ];
    gather::project_record(
        0,
        projection_input(&caches, &equality, &lslices, &options, 5),
    )
    .unwrap();
    assert_eq!(
        geometry_events(),
        [
            GeometryStep::TopAnchorOffset,
            GeometryStep::TopAnchorIntersection
        ]
    );

    reset_geometry_hooks();
    let below = vec![
        layer(0, 0.2, 0.0),
        layer(1, 0.2, f64::from_bits(boundary.to_bits() - 1)),
        layer(2, 0.2, 0.6),
    ];
    gather::project_record(0, projection_input(&caches, &below, &lslices, &options, 5)).unwrap();
    assert_eq!(
        geometry_events(),
        [GeometryStep::TopVisit, GeometryStep::HoleIntersection]
    );
}

#[test]
fn task22o20_bottom_window_uses_f64_bottom_z_subtraction() {
    let caches = vec![cache(0), cache(0), cache(0)];
    let lslices = vec![lslice(-100, 100); 3];
    let mut options = options();
    options.top_shell_layers.0 = 0;
    options.bottom_shell_layers.0 = 1;
    options.bottom_shell_thickness.0 = 0.2001;
    let boundary = options.bottom_shell_thickness.0 - 1e-4;

    reset_geometry_hooks();
    let equality = vec![
        layer(0, 0.2, -1.0),
        layer(1, 0.0, -boundary),
        layer(2, 0.4, 0.4),
    ];
    gather::project_record(
        2,
        projection_input(&caches, &equality, &lslices, &options, 5),
    )
    .unwrap();
    assert_eq!(
        geometry_events(),
        [
            GeometryStep::BottomAnchorOffset,
            GeometryStep::BottomAnchorIntersection,
        ]
    );

    reset_geometry_hooks();
    let below = vec![
        layer(0, 0.2, -1.0),
        layer(1, 0.0, -f64::from_bits(boundary.to_bits() - 1)),
        layer(2, 0.4, 0.4),
    ];
    gather::project_record(2, projection_input(&caches, &below, &lslices, &options, 5)).unwrap();
    assert_eq!(
        geometry_events(),
        [GeometryStep::BottomVisit, GeometryStep::HoleIntersection]
    );
}

#[test]
fn task22o20_none_neighbor_is_visited_clears_holes_and_suppresses_anchor() {
    reset_geometry_hooks();
    let caches = vec![cache(0), None, cache(0)];
    let layers = vec![layer(0, 0.2, 0.2), layer(1, 0.2, 0.4), layer(2, 0.2, 0.6)];
    let lslices = vec![lslice(-100, 100); 3];
    let mut options = options();
    options.top_shell_layers.0 = 3;
    options.top_shell_thickness.0 = 0.0;
    options.bottom_shell_layers.0 = 0;
    let output =
        gather::project_record(0, projection_input(&caches, &layers, &lslices, &options, 5))
            .unwrap();
    assert!(output.holes.is_empty());
    assert!(!output.shell.is_empty());
    assert_eq!(
        geometry_events(),
        [GeometryStep::TopVisit, GeometryStep::TopVisit]
    );
}

#[test]
fn task22o20_zero_and_negative_counts_run_neither_scan_nor_anchor() {
    let caches = vec![cache(0), cache(0), cache(0)];
    let layers = vec![layer(0, 0.2, 0.2), layer(1, 0.2, 0.4), layer(2, 0.2, 0.6)];
    let lslices = vec![lslice(-100, 100); 3];
    for (top, bottom) in [(0, 0), (-1, -2)] {
        reset_geometry_hooks();
        let mut options = options();
        options.top_shell_layers.0 = top;
        options.bottom_shell_layers.0 = bottom;
        let output =
            gather::project_record(1, projection_input(&caches, &layers, &lslices, &options, 5))
                .unwrap();
        assert!(output.shell.is_empty());
        assert_eq!(output.holes.len(), 1);
        assert!(geometry_events().is_empty());
    }
}

#[test]
fn task22o20_inactive_modes_stage_empty_without_geometry() {
    let caches = vec![cache(0), cache(0)];
    let layers = vec![layer(0, 0.2, 0.2), layer(1, 0.2, 0.4)];
    let lslices = vec![lslice(-100, 100); 2];
    for mode in [
        ProcessEnsureVerticalShellThickness::None,
        ProcessEnsureVerticalShellThickness::CriticalOnly,
        ProcessEnsureVerticalShellThickness::Moderate,
    ] {
        reset_geometry_hooks();
        let mut options = options();
        options.ensure_vertical_shell_thickness = mode;
        let output = gather::project_record(
            0,
            projection_input(&caches, &layers, &lslices, &options, i64::MAX),
        )
        .unwrap();
        assert!(output.shell.is_empty());
        assert!(output.holes.is_empty());
        assert!(geometry_events().is_empty());
    }
}

#[test]
fn task22o20_missing_first_bottom_and_last_top_neighbors_do_not_anchor() {
    let caches = vec![cache(0), cache(0), cache(0)];
    let layers = vec![layer(0, 0.2, 0.2), layer(1, 0.2, 0.4), layer(2, 0.2, 0.6)];
    let lslices = vec![lslice(-100, 100); 3];

    reset_geometry_hooks();
    let mut bottom = options();
    bottom.top_shell_layers.0 = 0;
    bottom.bottom_shell_layers.0 = 3;
    gather::project_record(0, projection_input(&caches, &layers, &lslices, &bottom, 5)).unwrap();
    assert!(geometry_events().is_empty());

    reset_geometry_hooks();
    let mut top = options();
    top.top_shell_layers.0 = 3;
    top.bottom_shell_layers.0 = 0;
    gather::project_record(2, projection_input(&caches, &layers, &lslices, &top, 5)).unwrap();
    assert!(geometry_events().is_empty());
}
