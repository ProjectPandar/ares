use super::interface_ids;
use crate::project_slice::raft::grid::{RaftLayerKind, RaftLayerZ};

fn layer(bottom_z: f64, print_z: f64, kind: RaftLayerKind) -> RaftLayerZ {
    RaftLayerZ {
        bottom_z,
        print_z,
        height: print_z - bottom_z,
        kind,
    }
}

/// `SupportCommon.cpp:1391-1403`: a contacts-only layer takes the
/// interface id of the layer it rests on plus one; base layers never
/// advance the counter.
#[test]
fn contact_layer_follows_its_supporting_layer() {
    let grid = [
        layer(0.0, 0.2, RaftLayerKind::Base),
        layer(0.2, 0.5, RaftLayerKind::Contact),
    ];
    assert_eq!(interface_ids(&grid), [0, 1]);
}

/// Interface layers advance the counter; the contact layer then
/// continues from the interface below it.
#[test]
fn interface_layers_advance_and_contact_continues() {
    let grid = [
        layer(0.0, 0.2, RaftLayerKind::Base),
        layer(0.2, 0.4, RaftLayerKind::Base),
        layer(0.4, 0.6, RaftLayerKind::Interface),
        layer(0.6, 0.8, RaftLayerKind::Interface),
        layer(0.8, 1.0, RaftLayerKind::Contact),
    ];
    assert_eq!(interface_ids(&grid), [0, 0, 0, 1, 2]);
}

/// A single-layer raft is only the contact layer: nothing supports it,
/// so it keeps the running counter.
#[test]
fn lone_contact_layer_keeps_counter() {
    let grid = [layer(0.0, 0.2, RaftLayerKind::Contact)];
    assert_eq!(interface_ids(&grid), [0]);
}
