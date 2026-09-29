use super::{interface_ids, raft_layer_plans};
use crate::project_slice::raft::fill_params::{RaftFillInputs, RaftFillPattern, flow_spacing};
use crate::project_slice::raft::grid::{RaftLayerKind, RaftLayerZ};
use crate::project_slice::raft::polygons::RaftPolygons;

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

/// `SupportCommon.cpp:1450-1516`: the flange and interface layers use
/// `filler_interface` (`raft_interface_fill_pattern`), base layers use
/// `filler_support` (`base_fill_pattern`) with `with_sheath`, and the base
/// sheath offsets come from the flow at the raft layer's own height.
#[test]
fn plans_select_the_upstream_filler_and_sheath_per_layer() {
    let grid = [
        layer(0.0, 0.2, RaftLayerKind::Base),
        layer(0.2, 0.5, RaftLayerKind::Base),
        layer(0.5, 0.7, RaftLayerKind::Interface),
    ];
    let params = RaftFillInputs {
        support_flow_spacing: 0.337,
        interface_flow_spacing: 0.337,
        base_pattern_spacing: 2.5,
        interface_spacing: 0.0,
        support_angle_degrees: 0.0,
        base_raft_layers: 2,
        interface_raft_layers: 1,
        with_sheath: true,
        honeycomb_base: true,
        tree_double_wall_area: 7.0,
    }
    .derive();
    let plans = raft_layer_plans(
        &grid,
        &RaftPolygons::default(),
        &params,
        0.337,
        0.377,
        90.0,
        0.38,
        0.38,
        0.42,
        0.2,
    );
    let specs: Vec<_> = plans.iter().map(|plan| plan.spec).collect();
    assert_eq!(specs[0].pattern, RaftFillPattern::Rectilinear);
    assert!(specs[0].sheath);
    assert!(specs[0].no_sort);
    assert_eq!(specs[0].tree_double_wall_area, 7.0);
    assert_eq!(specs[1].pattern, RaftFillPattern::Honeycomb);
    assert!(specs[1].sheath);
    assert!(!specs[1].no_sort);
    assert_eq!(specs[1].spacing, 0.337);
    assert_eq!(specs[1].sheath_spacing, flow_spacing(0.38, 0.3));
    assert_eq!(specs[2].pattern, RaftFillPattern::Rectilinear);
    assert!(!specs[2].sheath);
}
