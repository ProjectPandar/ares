use crate::geometry::{Point, Polygon};
use crate::project_slice::perimeters::types::Flow;

use super::super::super::hierarchy::PerimeterGeneratorLoop;
use super::super::types::{LowerFlowRoute, PendingExtrusionRole, PendingLoopRole, RouteFlows};
use super::classify_roots;

fn polygon(id: i64) -> Polygon {
    Polygon::new(vec![
        Point::new(id, 0),
        Point::new(id + 1, 0),
        Point::new(id, 1),
    ])
}

fn loop_(id: i64, depth: u16, contour: bool, smaller: bool) -> PerimeterGeneratorLoop {
    PerimeterGeneratorLoop {
        polygon: polygon(id),
        is_contour: contour,
        is_smaller_width_perimeter: smaller,
        depth,
        children: Vec::new(),
    }
}

fn flow(width: f32, mm3_per_mm: f64) -> Flow {
    Flow {
        width,
        height: 0.2,
        spacing: 0.4,
        nozzle_diameter: 0.4,
        bridge: false,
        mm3_per_mm,
    }
}

fn flows() -> RouteFlows {
    RouteFlows {
        perimeter: flow(1.000_000_1, 11.000_000_000_000_002),
        external: flow(2.000_000_2, 22.000_000_000_000_004),
        smaller_external: flow(3.000_000_2, 33.000_000_000_000_01),
    }
}

#[test]
fn task22o5_classifies_source_roles_and_all_flow_routes_exactly() {
    let roots = vec![
        loop_(0, 0, true, false),
        loop_(10, 0, true, true),
        loop_(20, 1, true, true),
        loop_(30, 2, false, false),
    ];
    let seeds = classify_roots(&roots, flows());
    assert_eq!(
        seeds[0].extrusion_role,
        PendingExtrusionRole::ExternalPerimeter
    );
    assert_eq!(seeds[2].extrusion_role, PendingExtrusionRole::Perimeter);
    assert_eq!(seeds[3].loop_role, PendingLoopRole::Hole);
    assert_eq!(
        seeds.iter().map(|seed| seed.route).collect::<Vec<_>>(),
        [
            LowerFlowRoute::External,
            LowerFlowRoute::SmallerExternal,
            LowerFlowRoute::Internal,
            LowerFlowRoute::Internal,
        ]
    );
    assert_eq!(seeds[0].width, 2.000_000_2_f32);
    assert_eq!(seeds[0].mm3_per_mm, 22.000_000_000_000_004_f64);
    assert_eq!(seeds[1].width, 3.000_000_2_f32);
    assert_eq!(seeds[1].mm3_per_mm, 33.000_000_000_000_01_f64);
    assert_eq!(seeds[2].width, 1.000_000_1_f32);
    assert_eq!(seeds[2].mm3_per_mm, 11.000_000_000_000_002_f64);
}

#[test]
fn task22o5_internal_contour_checks_immediate_children_only() {
    let mut root = loop_(0, 0, true, false);
    let mut hole = loop_(10, 1, false, false);
    hole.children.push(loop_(20, 2, true, false));
    root.children.push(hole);
    let seeds = classify_roots(&[root], flows());
    assert_eq!(seeds[0].loop_role, PendingLoopRole::Internal);
    assert_eq!(seeds[0].children[0].loop_role, PendingLoopRole::Hole);
    assert_eq!(
        seeds[0].children[0].children[0].loop_role,
        PendingLoopRole::Internal
    );
}

#[test]
fn task22o5_contour_with_immediate_contour_child_is_default() {
    let mut root = loop_(0, 0, true, false);
    root.children.push(loop_(10, 1, true, false));
    let seeds = classify_roots(&[root], flows());
    assert_eq!(seeds[0].loop_role, PendingLoopRole::Default);
    assert_eq!(seeds[0].children[0].loop_role, PendingLoopRole::Internal);
}

#[test]
fn task22o5_preserves_root_and_child_source_order() {
    let mut first = loop_(1, 0, true, false);
    first.children = vec![loop_(2, 1, false, false), loop_(3, 1, false, false)];
    let seeds = classify_roots(&[first, loop_(4, 0, true, false)], flows());
    assert_eq!(seeds[0].polygon.points()[0], Point::new(1, 0));
    assert_eq!(seeds[0].children[0].polygon.points()[0], Point::new(2, 0));
    assert_eq!(seeds[0].children[1].polygon.points()[0], Point::new(3, 0));
    assert_eq!(seeds[1].polygon.points()[0], Point::new(4, 0));
}

#[test]
fn task22o5_classifies_deep_trees_iteratively() {
    std::thread::Builder::new()
        .stack_size(crate::project_slice::CONSTRAINED_TEST_STACK_SIZE)
        .spawn(|| {
            let mut root = loop_(20_000, 1, false, false);
            for id in (0..20_000).rev() {
                let mut parent = loop_(id, 1, false, false);
                parent.children.push(root);
                root = parent;
            }
            let mut sources = vec![root];
            let mut seeds = classify_roots(&sources, flows());
            let mut source_pending = sources.split_off(0);
            while let Some(mut node) = source_pending.pop() {
                source_pending.append(&mut node.children);
            }
            let mut count = 0;
            while let Some(mut seed) = seeds.pop() {
                count += 1;
                seeds.append(&mut seed.children);
            }
            assert_eq!(count, 20_001);
        })
        .unwrap()
        .join()
        .unwrap();
}
