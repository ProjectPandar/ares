use super::{
    super::{
        chained_loops::{ExtrusionLoop, ExtrusionLoopRole},
        entity_collections::{ExtrusionEntityCollection, OrderedExtrusionLoop},
        materialize::{ExtrusionPath, ExtrusionRole, Point3, Polyline3},
        traversal::{ClassicTraversalRecord, InactiveOverhangReverse, PendingPathBranch},
    },
    InactiveOuterBrimReordering, InactiveOverhangReorientation, InactivePostCollectionBranches,
    InactiveWallReordering, append_nonempty, classify_inactive,
};
use crate::{
    ObjectOptions, OrcaBool, OrcaFloat, ProcessBrimType, ProjectSettings, RegionOptions,
    project_slice::perimeters::types::Flow,
};

#[test]
fn task22o10_empty_collection_is_not_appended() {
    assert!(
        append_nonempty(ExtrusionEntityCollection::default())
            .collections
            .is_empty()
    );
}

#[test]
fn task22o10_nonempty_collection_keeps_nested_boundary_order_and_allocations() {
    let collection = collection();
    let allocation = collection.entities[0].extrusion_loop.paths[0]
        .polyline
        .points
        .as_ptr();
    let appended = append_nonempty(collection);
    assert_eq!(appended.collections.len(), 1);
    assert_eq!(appended.collections[0].entities.len(), 2);
    assert_eq!(appended.collections[0].entities[0].inset_idx, 0);
    assert_eq!(appended.collections[0].entities[1].inset_idx, 1);
    assert_eq!(
        appended.collections[0].entities[0].extrusion_loop.paths[0]
            .polyline
            .points
            .as_ptr(),
        allocation
    );
}

#[test]
fn task22o10_inactive_provenance_exhausts_accepted_outer_brim_reasons() {
    let mut region = RegionOptions::from_base(&ProjectSettings::default().process.region);
    region.overhang_reverse_internal_only = OrcaBool(true);
    let mut object = ObjectOptions::from_base(&ProjectSettings::default().process.object);

    object.brim_type = ProcessBrimType::OuterOnly;
    object.brim_width = OrcaFloat(5.0);
    let (internal_only, reason) = inactive_parts(classify_inactive(&record(1), &region, &object));
    assert!(internal_only);
    assert!(matches!(
        reason,
        InactiveOuterBrimReordering::LaterLayer {
            layer_id: 1,
            brim_type: ProcessBrimType::OuterOnly,
            brim_width: 5.0,
        }
    ));

    object.brim_type = ProcessBrimType::AutoBrim;
    let (_, reason) = inactive_parts(classify_inactive(&record(0), &region, &object));
    assert!(matches!(
        reason,
        InactiveOuterBrimReordering::DifferentBrimType {
            brim_type: ProcessBrimType::AutoBrim,
            brim_width: 5.0,
        }
    ));

    object.brim_type = ProcessBrimType::OuterOnly;
    object.brim_width = OrcaFloat(0.0);
    let (_, reason) = inactive_parts(classify_inactive(&record(0), &region, &object));
    assert!(matches!(
        reason,
        InactiveOuterBrimReordering::WidthNotPositive { brim_width: 0.0 }
    ));
}

fn inactive_parts(inactive: InactivePostCollectionBranches) -> (bool, InactiveOuterBrimReordering) {
    let InactiveOverhangReorientation::Disabled {
        overhang_reverse_internal_only,
    } = inactive.overhang_reorientation;
    let InactiveWallReordering::InnerOuter { outer_brim } = inactive.wall_reordering;
    (overhang_reverse_internal_only, outer_brim)
}

fn record(layer_id: usize) -> ClassicTraversalRecord {
    ClassicTraversalRecord {
        surfaces: Vec::new(),
        layer_height: 0.2,
        overhang_flow: flow(),
        branch: PendingPathBranch::OrdinaryUnsplit {
            detect_overhang_wall: false,
            layer_id,
            raft_layers: 0,
        },
        overhang_reverse: InactiveOverhangReverse {
            configured: false,
            odd_layer: layer_id % 2 == 1,
            active: false,
        },
    }
}

fn flow() -> Flow {
    Flow {
        width: 0.4,
        height: 0.2,
        spacing: 0.36,
        nozzle_diameter: 0.4,
        bridge: false,
        mm3_per_mm: 0.08,
    }
}

fn collection() -> ExtrusionEntityCollection {
    ExtrusionEntityCollection {
        entities: vec![entity(0, 0), entity(10, 1)],
    }
}

fn entity(x: i64, inset_idx: i32) -> OrderedExtrusionLoop {
    OrderedExtrusionLoop {
        extrusion_loop: ExtrusionLoop {
            paths: vec![ExtrusionPath {
                polyline: Polyline3 {
                    points: vec![
                        Point3 { x, y: 0, z: 0 },
                        Point3 {
                            x: x + 4,
                            y: 0,
                            z: 0,
                        },
                        Point3 { x, y: 0, z: 0 },
                    ],
                    fitting: Vec::new(),
                    candidate_points: Vec::new(),
                },
                role: ExtrusionRole::Perimeter,
                can_reverse: true,
                mm3_per_mm: 0.08,
                width: 0.4,
                height: 0.2,
            }],
            role: ExtrusionLoopRole::Default,
        },
        inset_idx,
    }
}
