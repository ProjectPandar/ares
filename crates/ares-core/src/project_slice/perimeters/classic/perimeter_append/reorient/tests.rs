use super::ReorientPlan;
use crate::geometry::{CoordinateScale, Point, Polygon};
use crate::{
    FloatOrPercent, OrcaBool, Percent, ProjectSettings, RegionOptions,
    project_slice::perimeters::types::Flow,
};

use super::super::super::{
    chained_loops::{ExtrusionLoop, ExtrusionLoopRole},
    entity_collections::{ExtrusionEntity, ExtrusionEntityCollection, OrderedExtrusionLoop},
    materialize::{ExtrusionPath, ExtrusionRole, Point3, Polyline3},
    traversal::{
        ClassicTraversalRecord, LowerFlowRoute, PendingExtrusionRole, PendingLoopRole,
        PendingPathBranch, TraversalSeed,
    },
};

fn region(threshold: f64) -> RegionOptions {
    let mut region = RegionOptions::from_base(&ProjectSettings::default().process.region);
    region.overhang_reverse = OrcaBool(true);
    region.overhang_reverse_internal_only = OrcaBool(true);
    region.overhang_reverse_threshold = FloatOrPercent::Percent(Percent(threshold));
    region
}

fn record(layer_id: usize, raft_layers: i32, detect: bool) -> ClassicTraversalRecord {
    ClassicTraversalRecord {
        surfaces: Vec::new(),
        layer_id,
        layer_height: 0.2,
        slice_z: layer_id as f64 * 0.2 + 0.1,
        fuzzy_skin: crate::project_slice::perimeters::arachne::fuzzy::EMPTY.clone(),
        overhang_flow: Flow {
            width: 0.4,
            height: 0.2,
            spacing: 0.36,
            nozzle_diameter: 0.4,
            bridge: false,
            mm3_per_mm: 0.08,
        },
        ext_perimeter_flow: Flow {
            width: 0.45,
            height: 0.2,
            spacing: 0.45,
            nozzle_diameter: 0.4,
            bridge: false,
            mm3_per_mm: 0.09,
        },
        scale: CoordinateScale::Normal,
        branch: if detect {
            PendingPathBranch::OverhangClipping {
                detect_overhang_wall: true,
                layer_id,
                raft_layers,
            }
        } else {
            PendingPathBranch::OrdinaryUnsplit {
                detect_overhang_wall: false,
                layer_id,
                raft_layers,
            }
        },
    }
}

fn seed(is_contour: bool) -> TraversalSeed {
    TraversalSeed {
        polygon: Polygon::new(vec![
            Point::new(0, 0),
            Point::new(1_000_000, 0),
            Point::new(1_000_000, 1_000_000),
            Point::new(0, 1_000_000),
        ]),
        depth: 0,
        is_contour,
        is_smaller_width_perimeter: false,
        extrusion_role: if is_contour {
            PendingExtrusionRole::ExternalPerimeter
        } else {
            PendingExtrusionRole::Perimeter
        },
        loop_role: if is_contour {
            PendingLoopRole::Default
        } else {
            PendingLoopRole::Hole
        },
        route: LowerFlowRoute::External,
        width: 0.42,
        mm3_per_mm: 0.08,
        children: Vec::new(),
    }
}

fn loop_entity(role: ExtrusionLoopRole, path_role: ExtrusionRole, x: i64) -> ExtrusionEntity {
    ExtrusionEntity::Loop(OrderedExtrusionLoop {
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
                        Point3 {
                            x: x + 9,
                            y: 0,
                            z: 0,
                        },
                    ],
                    fitting: Vec::new(),
                },
                role: path_role,
                can_reverse: true,
                mm3_per_mm: 0.08,
                width: 0.4,
                height: 0.2,
            }],
            role,
        },
        inset_idx: 0,
    })
}

fn first_points(collection: &ExtrusionEntityCollection, index: usize) -> Vec<i64> {
    match &collection.entities[index] {
        ExtrusionEntity::Loop(ordered) => ordered
            .extrusion_loop
            .paths
            .iter()
            .flat_map(|path| path.polyline.points.iter().map(|point| point.x))
            .collect(),
        ExtrusionEntity::MultiPath(_) | ExtrusionEntity::Path(_) => {
            panic!("classic append keeps loop entities")
        }
    }
}

#[test]
fn zero_threshold_odd_layer_reverses_inner_and_exempts_external() {
    let seeds = vec![seed(true)];
    let mut collection = ExtrusionEntityCollection {
        entities: vec![
            loop_entity(
                ExtrusionLoopRole::Default,
                ExtrusionRole::ExternalPerimeter,
                30,
            ),
            loop_entity(ExtrusionLoopRole::Default, ExtrusionRole::Perimeter, 10),
        ],
        source_order: 0,
    };
    ReorientPlan::new(&record(1, 0, true), &region(0.0)).apply(
        &mut collection,
        &seeds,
        None,
        CoordinateScale::Normal,
    );
    // `reorient_perimeters` exempts the external loop
    // (`PerimeterGenerator.cpp:1129-1135`) and reverses the internal one.
    assert_eq!(first_points(&collection, 0), vec![30, 34, 39]);
    assert_eq!(first_points(&collection, 1), vec![19, 14, 10]);
}

#[test]
fn even_layer_keeps_orientation() {
    let seeds = vec![seed(true)];
    let mut collection = ExtrusionEntityCollection {
        entities: vec![loop_entity(
            ExtrusionLoopRole::Default,
            ExtrusionRole::Perimeter,
            10,
        )],
        source_order: 0,
    };
    ReorientPlan::new(&record(0, 0, true), &region(0.0)).apply(
        &mut collection,
        &seeds,
        None,
        CoordinateScale::Normal,
    );
    assert_eq!(first_points(&collection, 0), vec![10, 14, 19]);
}

#[test]
fn ordinary_unsplit_above_raft_reverses_internal_only() {
    let seeds = vec![seed(true)];
    let mut collection = ExtrusionEntityCollection {
        entities: vec![
            loop_entity(
                ExtrusionLoopRole::Default,
                ExtrusionRole::ExternalPerimeter,
                30,
            ),
            loop_entity(ExtrusionLoopRole::Default, ExtrusionRole::Perimeter, 10),
        ],
        source_order: 0,
    };
    ReorientPlan::new(&record(3, 0, false), &region(50.0)).apply(
        &mut collection,
        &seeds,
        None,
        CoordinateScale::Normal,
    );
    assert_eq!(first_points(&collection, 0), vec![30, 34, 39]);
    assert_eq!(first_points(&collection, 1), vec![19, 14, 10]);
}

#[test]
fn hole_loop_follows_hole_flag() {
    let seeds = vec![seed(false)];
    let mut collection = ExtrusionEntityCollection {
        entities: vec![
            loop_entity(ExtrusionLoopRole::Hole, ExtrusionRole::Perimeter, 10),
            loop_entity(ExtrusionLoopRole::Default, ExtrusionRole::Perimeter, 20),
        ],
        source_order: 0,
    };
    ReorientPlan::new(&record(1, 0, true), &region(0.0)).apply(
        &mut collection,
        &seeds,
        None,
        CoordinateScale::Normal,
    );
    // Only the steep-hole flag is set, so the default-role loop stays.
    assert_eq!(first_points(&collection, 0), vec![19, 14, 10]);
    assert_eq!(first_points(&collection, 1), vec![20, 24, 29]);
}
