use super::fixture::options;
use crate::project_slice::{
    layers::PlannedLayer,
    prepare_infill::horizontal_shell_propagation::{types::SourceKind, window},
};

fn layer(id: usize, print_z: f64, height: f64) -> PlannedLayer {
    PlannedLayer {
        id,
        height,
        print_z,
        slice_z: print_z - height / 2.0,
    }
}

#[test]
fn neighbor_indices_are_top_down_and_bottom_kinds_up() {
    assert!(!window::source_enabled(0));
    assert!(window::source_enabled(-1));
    assert!(window::source_enabled(1));
    assert_eq!(
        window::indices(SourceKind::Top, 3, 6).collect::<Vec<_>>(),
        vec![2, 1, 0]
    );
    assert_eq!(
        window::indices(SourceKind::Bottom, 2, 6).collect::<Vec<_>>(),
        vec![3, 4, 5]
    );
    assert_eq!(
        window::indices(SourceKind::BottomBridge, 2, 6).collect::<Vec<_>>(),
        vec![3, 4, 5]
    );
}

#[test]
fn windows_use_count_or_strict_thickness_and_ignore_stored_ids() {
    let layers = [
        layer(90, 0.25, 0.25),
        layer(7, 0.5, 0.25),
        layer(500, 0.75, 0.25),
    ];
    let mut top = options();
    top.top_shell_layers.0 = 2;
    top.top_shell_thickness.0 = 0.0;
    assert!(window::includes(SourceKind::Top, [2, 1], &layers, 2, &top));
    assert!(!window::includes(SourceKind::Top, [2, 0], &layers, 2, &top));

    top.top_shell_layers.0 = -1;
    top.top_shell_thickness.0 = 0.5002;
    assert!(window::includes(SourceKind::Top, [2, 0], &layers, -1, &top));
    top.top_shell_thickness.0 = 0.5001;
    assert!(!window::includes(
        SourceKind::Top,
        [2, 0],
        &layers,
        -1,
        &top
    ));
}

#[test]
fn bottom_bridge_uses_bottom_count_and_variable_bottom_z_thickness() {
    let layers = [
        layer(0, 0.25, 0.25),
        layer(1, 0.5, 0.25),
        layer(2, 0.75, 0.25),
    ];
    let mut options = options();
    options.top_shell_layers.0 = 99;
    options.bottom_shell_layers.0 = -3;
    options.bottom_shell_thickness.0 = 0.5002;
    assert_eq!(window::shell_count(SourceKind::BottomBridge, &options), -3);
    assert!(window::includes(
        SourceKind::BottomBridge,
        [0, 2],
        &layers,
        -3,
        &options,
    ));
    options.bottom_shell_thickness.0 = 0.5001;
    assert!(!window::includes(
        SourceKind::Bottom,
        [0, 2],
        &layers,
        -3,
        &options,
    ));
}
