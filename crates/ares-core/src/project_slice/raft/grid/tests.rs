use super::*;

fn raft_parameters(
    base_layers: usize,
    interface_layers: usize,
    base_height: f64,
    interface_height: f64,
    contact_height: f64,
    first_print_layer_height: f64,
) -> crate::project_slice::parameters::SlicingParameters {
    use crate::project_slice::parameters::SlicingParameters;
    SlicingParameters {
        base_raft_layers: base_layers,
        interface_raft_layers: interface_layers,
        base_raft_layer_height: base_height,
        interface_raft_layer_height: interface_height,
        contact_raft_layer_height: contact_height,
        layer_height: 0.2,
        min_layer_height: 0.07,
        max_layer_height: 0.28,
        first_print_layer_height,
        first_object_layer_height: 0.2,
        first_object_layer_bridging: false,
        gap_raft_object: 0.2,
        gap_object_support: 0.2,
        gap_support_object: 0.2,
        raft_base_top_z: first_print_layer_height + (base_layers.max(1) - 1) as f64 * base_height,
        raft_interface_top_z: first_print_layer_height
            + (base_layers.max(1) - 1) as f64 * base_height
            + (interface_layers.max(1) - 1) as f64 * interface_height,
        raft_contact_top_z: first_print_layer_height
            + (base_layers.max(1) - 1) as f64 * base_height
            + (interface_layers.max(1) - 1) as f64 * interface_height
            + contact_height,
        object_print_z_min: first_print_layer_height
            + (base_layers.max(1) - 1) as f64 * base_height
            + (interface_layers.max(1) - 1) as f64 * interface_height
            + contact_height
            + 0.2,
        object_print_z_max: 10.3,
        object_print_z_uncompensated_max: 10.1,
        object_shrinkage_compensation_z: 1.0,
    }
}

/// KSR sweep oracle `option/raft_layers/max` (=100, 0.4 nozzle, 0.2
/// layer height): the 150 `;LAYER_CHANGE` z sequence is
/// 0.2, then 0.3 steps to 14.9 (first + 49 base), 0.3 steps to 29.6
/// (49 interface), contact 29.9, object from 30.1 (`Slicing.cpp:194`
/// split: base=50, interface=50).
#[test]
fn grid_matches_oracle_raft_layers_100() {
    let parameters = raft_parameters(50, 50, 0.3, 0.3, 0.3, 0.2);
    let grid = raft_layer_grid(&parameters).unwrap();

    assert_eq!(grid.len(), 100);
    let z_values = grid.iter().map(|layer| layer.print_z).collect::<Vec<_>>();
    let mut expected = vec![0.2];
    let mut z = 0.2;
    for _ in 0..49 {
        z += 0.3;
        expected.push(round6(z));
    }
    for _ in 0..49 {
        z += 0.3;
        expected.push(round6(z));
    }
    expected.push(round6(z + 0.3));
    for (actual, expected) in z_values.iter().zip(&expected) {
        assert!(
            (actual - expected).abs() < 1e-9,
            "z mismatch: {actual} vs {expected}"
        );
    }
    assert_eq!(grid[0].kind, RaftLayerKind::Base);
    assert_eq!(grid[0].bottom_z, 0.0);
    assert_eq!(grid[49].kind, RaftLayerKind::Base);
    assert_eq!(grid[50].kind, RaftLayerKind::Interface);
    assert_eq!(grid[98].kind, RaftLayerKind::Interface);
    assert_eq!(grid[99].kind, RaftLayerKind::Contact);
    assert!((grid[99].print_z - 29.9).abs() < 1e-9);
    // The object starts above the contact layer plus the raft gap.
    assert!((parameters.object_print_z_min - 30.1).abs() < 1e-9);
}

fn round6(value: f64) -> f64 {
    (value * 1e6).round() / 1e6
}

/// `Slicing.cpp:195-196` split: interface = (N+1)/2, base = N −
/// interface; `raft_layers()==1` degenerates to a single contact
/// layer at `initial_layer_print_height` (`Slicing.cpp:208-211`).
#[test]
fn single_raft_layer_is_contact_only() {
    let parameters = raft_parameters(0, 1, 0.3, 0.3, 0.2, 0.2);
    let grid = raft_layer_grid(&parameters).unwrap();

    assert_eq!(grid.len(), 1);
    assert_eq!(grid[0].kind, RaftLayerKind::Contact);
    assert_eq!(grid[0].print_z, 0.2);
    assert_eq!(grid[0].bottom_z, 0.0);
}

#[test]
fn no_raft_yields_empty_grid() {
    let parameters = raft_parameters(0, 0, 0.0, 0.0, 0.0, 0.2);
    assert!(raft_layer_grid(&parameters).unwrap().is_empty());
}

/// Heights follow each layer's own step (`Slicing.cpp:200-202`): the
/// base uses `base_raft_layer_height` while the contact uses
/// `contact_raft_layer_height`, which differ when the interface
/// extruder differs.
#[test]
fn mixed_heights_step_per_kind() {
    let parameters = raft_parameters(2, 2, 0.3, 0.24, 0.24, 0.2);
    let grid = raft_layer_grid(&parameters).unwrap();

    assert_eq!(grid.len(), 4);
    let z_values = grid.iter().map(|layer| layer.print_z).collect::<Vec<_>>();
    for (actual, expected) in z_values.iter().zip([0.2, 0.5, 0.74, 0.98]) {
        assert!((actual - expected).abs() < 1e-9);
    }
    assert_eq!(grid[2].kind, RaftLayerKind::Interface);
    assert_eq!(grid[3].kind, RaftLayerKind::Contact);
}
