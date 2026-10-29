use super::{
    super::{
        chained_loops::{ExtrusionLoop, ExtrusionLoopRole},
        entity_collections::{ExtrusionEntity, ExtrusionEntityCollection, OrderedExtrusionLoop},
        materialize::{ExtrusionPath, ExtrusionRole, Point3, Polyline3},
        traversal::{ClassicTraversalRecord, PendingPathBranch},
    },
    InactiveOuterBrimReordering, InactivePostCollectionBranches, InactiveWallReordering,
    append_nonempty, classify_inactive, reorder_walls,
};
use crate::{
    ObjectOptions, OrcaFloat, ProcessBrimType, ProcessWallSequence, ProjectSettings,
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
fn wall_sequences_reorder_inset_indices_like_orca() {
    let source = || ExtrusionEntityCollection {
        entities: vec![
            ExtrusionEntity::Loop(entity(30, 3)),
            ExtrusionEntity::Loop(entity(20, 2)),
            ExtrusionEntity::Loop(entity(10, 1)),
            ExtrusionEntity::Loop(entity(0, 0)),
        ],
        source_order: 0,
    };
    let indices = |collection: &ExtrusionEntityCollection| {
        collection
            .entities
            .iter()
            .map(|entity| entity.inset_idx())
            .collect::<Vec<_>>()
    };

    let mut outer_inner = source();
    reorder_walls(&mut outer_inner, ProcessWallSequence::OuterInner, 1);
    assert_eq!(indices(&outer_inner), [0, 1, 2, 3]);

    let mut sandwich = source();
    reorder_walls(&mut sandwich, ProcessWallSequence::InnerOuterInner, 1);
    assert_eq!(indices(&sandwich), [3, 2, 0, 1]);

    let mut first_layer = source();
    reorder_walls(&mut first_layer, ProcessWallSequence::InnerOuterInner, 0);
    assert_eq!(indices(&first_layer), [3, 2, 1, 0]);
}

#[test]
fn outer_inner_reverses_thin_path_direction() {
    // `ExtrusionEntityCollection::reverse` flips a non-loop Path's direction
    // (upstream `ExtrusionEntityCollection.cpp:65-72`), not just the order.
    let mut collection = ExtrusionEntityCollection {
        entities: vec![thin_path(0, 10)],
        source_order: 0,
    };
    reorder_walls(&mut collection, ProcessWallSequence::OuterInner, 1);
    let ExtrusionEntity::Path(path) = &collection.entities[0] else {
        panic!("thin path stays a Path");
    };
    assert_eq!(
        path.polyline.points[0].x, 10,
        "reversed path starts at far end"
    );
    assert_eq!(path.polyline.points[1].x, 0);
}

#[test]
fn inner_outer_inner_handles_leading_thin_path() {
    // `[Loop2,Loop1,Loop0,Path(-1)]` on layer>0: a leading thin path must not
    // break the island scan — upstream produces `[Loop2, Path, Loop0, Loop1]`
    // via its reorder span (`PerimeterGenerator.cpp:1483-1560`).
    let mut collection = ExtrusionEntityCollection {
        entities: vec![
            ExtrusionEntity::Loop(entity(30, 2)),
            ExtrusionEntity::Loop(entity(20, 1)),
            ExtrusionEntity::Loop(entity(10, 0)),
            thin_path(5, 6),
        ],
        source_order: 0,
    };
    reorder_walls(&mut collection, ProcessWallSequence::InnerOuterInner, 1);
    let kinds: Vec<i32> = collection.entities.iter().map(|e| e.inset_idx()).collect();
    // Upstream `inner_outer_extrusions` emits the deepest internal first
    // (Loop2 at max_internal), then the outside-in head — `[2, -1, 0, 1]`
    // (`PerimeterGenerator.cpp:1536-1548`).
    assert_eq!(kinds, [2, -1, 0, 1]);
    // The thin path also flips direction under `reverse_collection` (5->6
    // becomes 6->5).
    let ExtrusionEntity::Path(path) = &collection.entities[1] else {
        panic!("second entity is the thin path");
    };
    assert_eq!(path.polyline.points[0].x, 6);
    assert_eq!(path.polyline.points[1].x, 5);
}

#[test]
fn inner_outer_inner_incomplete_tail_keeps_reversed_order() {
    // `[1,3,0,2,1,0]` reversed -> `[0,1,2,0,3,1]`; phase-1 -> `[0,1,2,0,1,3]`.
    // The trailing island `[0,1,3]` has no inset 2, so upstream leaves the
    // ORIGINAL reversed tail `[0,3,1]` — not the phase-1 order `[0,1,3]`.
    let mut collection = ExtrusionEntityCollection {
        entities: vec![
            ExtrusionEntity::Loop(entity(1, 1)),
            ExtrusionEntity::Loop(entity(3, 3)),
            ExtrusionEntity::Loop(entity(0, 0)),
            ExtrusionEntity::Loop(entity(2, 2)),
            ExtrusionEntity::Loop(entity(11, 1)),
            ExtrusionEntity::Loop(entity(10, 0)),
        ],
        source_order: 0,
    };
    reorder_walls(&mut collection, ProcessWallSequence::InnerOuterInner, 1);
    let kinds: Vec<i32> = collection.entities.iter().map(|e| e.inset_idx()).collect();
    assert_eq!(kinds, [2, 0, 1, 0, 3, 1]);
}

#[test]
fn task22o10_nonempty_collection_keeps_nested_boundary_order_and_allocations() {
    let collection = collection();
    let allocation = collection_loop(&collection, 0).paths[0]
        .polyline
        .points
        .as_ptr();
    let appended = append_nonempty(collection);
    assert_eq!(appended.collections.len(), 1);
    assert_eq!(appended.collections[0].entities.len(), 2);
    assert_eq!(appended.collections[0].entities[0].inset_idx(), 0);
    assert_eq!(appended.collections[0].entities[1].inset_idx(), 1);
    assert_eq!(
        collection_loop(&appended.collections[0], 0).paths[0]
            .polyline
            .points
            .as_ptr(),
        allocation
    );
}

#[test]
fn task22o10_inactive_provenance_exhausts_accepted_outer_brim_reasons() {
    let mut object = ObjectOptions::from_base(&ProjectSettings::default().process.object);

    object.brim_type = ProcessBrimType::OuterOnly;
    object.brim_width = OrcaFloat(5.0);
    let reason = inactive_parts(classify_inactive(&record(1), &object));
    assert!(matches!(
        reason,
        InactiveOuterBrimReordering::LaterLayer {
            layer_id: 1,
            brim_type: ProcessBrimType::OuterOnly,
            brim_width: 5.0,
        }
    ));

    object.brim_type = ProcessBrimType::AutoBrim;
    let reason = inactive_parts(classify_inactive(&record(0), &object));
    assert!(matches!(
        reason,
        InactiveOuterBrimReordering::DifferentBrimType {
            brim_type: ProcessBrimType::AutoBrim,
            brim_width: 5.0,
        }
    ));

    object.brim_type = ProcessBrimType::OuterOnly;
    object.brim_width = OrcaFloat(0.0);
    let reason = inactive_parts(classify_inactive(&record(0), &object));
    assert!(matches!(
        reason,
        InactiveOuterBrimReordering::WidthNotPositive { brim_width: 0.0 }
    ));
}

fn inactive_parts(inactive: InactivePostCollectionBranches) -> InactiveOuterBrimReordering {
    let InactiveWallReordering::InnerOuter { outer_brim } = inactive.wall_reordering;
    outer_brim
}

fn record(layer_id: usize) -> ClassicTraversalRecord {
    ClassicTraversalRecord {
        surfaces: Vec::new(),
        layer_id,
        layer_height: 0.2,
        slice_z: layer_id as f64 * 0.2 + 0.1,
        fuzzy_skin: crate::project_slice::perimeters::arachne::fuzzy::EMPTY.clone(),
        overhang_flow: flow(),
        ext_perimeter_flow: flow(),
        scale: crate::geometry::CoordinateScale::Normal,
        branch: PendingPathBranch::OrdinaryUnsplit {
            detect_overhang_wall: false,
            layer_id,
            raft_layers: 0,
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

fn collection_loop(collection: &ExtrusionEntityCollection, index: usize) -> &ExtrusionLoop {
    match &collection.entities[index] {
        ExtrusionEntity::Loop(ordered) => &ordered.extrusion_loop,
        ExtrusionEntity::MultiPath(_) | ExtrusionEntity::Path(_) => {
            panic!("classic append keeps loop entities")
        }
    }
}

fn collection() -> ExtrusionEntityCollection {
    ExtrusionEntityCollection {
        entities: vec![
            ExtrusionEntity::Loop(entity(0, 0)),
            ExtrusionEntity::Loop(entity(10, 1)),
        ],
        source_order: 0,
    }
}

fn thin_path(x0: i64, x1: i64) -> ExtrusionEntity {
    ExtrusionEntity::Path(ExtrusionPath {
        z_contoured: false,
        polyline: Polyline3 {
            points: vec![Point3 { x: x0, y: 0, z: 0 }, Point3 { x: x1, y: 0, z: 0 }],
            fitting: Vec::new(),
        },
        role: ExtrusionRole::ExternalPerimeter,
        can_reverse: true,
        mm3_per_mm: 0.08,
        width: 0.4,
        height: 0.2,
    })
}

fn entity(x: i64, inset_idx: i32) -> OrderedExtrusionLoop {
    OrderedExtrusionLoop {
        unsupported_distance: None,
        extrusion_loop: ExtrusionLoop {
            paths: vec![ExtrusionPath {
                z_contoured: false,
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
