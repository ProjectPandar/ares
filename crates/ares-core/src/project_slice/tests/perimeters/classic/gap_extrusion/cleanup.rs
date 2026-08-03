use crate::{
    geometry::{ExPolygon, Point, Polygon},
    project_slice::{
        incomplete_sink,
        perimeters::{
            classic::{
                gap_extrusion,
                hierarchy::PerimeterGeneratorLoop,
                medial_gap::PreparedPostClassicMedialGap,
                traversal::{PreparedPostClassicTraversal, TraversalSeed},
            },
            prepare_post_classic_medial_gap,
        },
    },
};

use super::super::super::super::support::ksr_project;
use super::aligned::source_with_filter;

#[test]
fn task22o14_success_and_every_error_cleanup_fit_constrained_stack() {
    let mut success =
        gap_extrusion::finish(prepare_post_classic_medial_gap(ksr_project()).unwrap()).unwrap();
    deepen_both_tree_families(&mut success.predecessor);
    run_on_constrained_stack(move || {
        for object in success.objects {
            incomplete_sink::consume_gap_extrusion_object(object);
        }
        incomplete_sink::consume_boxed_post_classic_traversal(success.predecessor);
    });

    run_failure_on_constrained_stack(source_with_filter("-1"));

    let mut invalid_flow = source_with_filter("0");
    inject_invalid_flow(&mut invalid_flow);
    run_failure_on_constrained_stack(invalid_flow);

    let mut invalid_offset = source_with_filter("0");
    inject_invalid_offset(&mut invalid_offset);
    run_failure_on_constrained_stack(invalid_offset);

    let mut invalid_difference = source_with_filter("0");
    inject_invalid_difference(&mut invalid_difference);
    run_failure_on_constrained_stack(invalid_difference);
}

fn run_failure_on_constrained_stack(mut failure: PreparedPostClassicMedialGap) {
    deepen_both_tree_families(&mut failure.predecessor);
    run_on_constrained_stack(move || {
        assert!(gap_extrusion::finish(failure).is_err());
    });
}

fn inject_invalid_flow(source: &mut PreparedPostClassicMedialGap) {
    let (object, record, _) = retained_surface(source);
    let prelude = &mut source.predecessor.objects[object]
        .predecessor
        .predecessor
        .predecessor
        .predecessor;
    prelude.object.records[record]
        .as_mut()
        .unwrap()
        .solid_infill_flow
        .height = f32::NAN;
}

fn inject_invalid_offset(source: &mut PreparedPostClassicMedialGap) {
    let (object, record, surface) = retained_surface(source);
    let polyline = &mut source.objects[object].records[record]
        .as_mut()
        .unwrap()
        .surfaces[surface]
        .medial
        .as_mut()
        .unwrap()
        .polylines[0];
    polyline.points = vec![
        Point::new(i64::MAX - 1_000, 0),
        Point::new(i64::MAX - 500, 0),
    ];
    polyline.width = vec![400_000.0, 400_000.0];
}

fn inject_invalid_difference(source: &mut PreparedPostClassicMedialGap) {
    let (object, record, surface) = retained_surface(source);
    source.predecessor.objects[object]
        .predecessor
        .predecessor
        .records[record]
        .as_mut()
        .unwrap()
        .surfaces[surface]
        .last = vec![ExPolygon::new(
        Polygon::new(vec![
            Point::new(i64::MAX - 20, 0),
            Point::new(i64::MAX - 10, 0),
            Point::new(i64::MAX - 10, 10),
            Point::new(i64::MAX - 20, 10),
        ]),
        Vec::new(),
    )];
}

fn retained_surface(source: &PreparedPostClassicMedialGap) -> (usize, usize, usize) {
    source
        .objects
        .iter()
        .enumerate()
        .find_map(|(object_index, object)| {
            object
                .records
                .iter()
                .enumerate()
                .find_map(|(record_index, record)| {
                    record.as_ref()?.surfaces.iter().enumerate().find_map(
                        |(surface_index, surface)| {
                            surface
                                .medial
                                .as_ref()
                                .is_some_and(|domain| !domain.polylines.is_empty())
                                .then_some((object_index, record_index, surface_index))
                        },
                    )
                })
        })
        .expect("KSR must contain a retained medial polyline")
}

fn run_on_constrained_stack(action: impl FnOnce() + Send + 'static) {
    std::thread::Builder::new()
        .stack_size(64 * 1024)
        .spawn(action)
        .unwrap()
        .join()
        .unwrap();
}

fn deepen_both_tree_families(prepared: &mut PreparedPostClassicTraversal) {
    let traversal = prepared
        .objects
        .iter_mut()
        .find(|object| {
            object.records.iter().flatten().any(|record| {
                record
                    .surfaces
                    .iter()
                    .any(|surface| !surface.roots.is_empty())
            })
        })
        .unwrap();
    let roots = &mut traversal
        .records
        .iter_mut()
        .flatten()
        .flat_map(|record| &mut record.surfaces)
        .find(|surface| !surface.roots.is_empty())
        .unwrap()
        .roots;
    *roots = vec![deep_seed(roots.first().unwrap(), 10_000)];

    let hierarchy_roots = &mut traversal
        .predecessor
        .records
        .iter_mut()
        .flatten()
        .flat_map(|record| &mut record.surfaces)
        .find(|surface| !surface.roots.is_empty())
        .unwrap()
        .roots;
    *hierarchy_roots = vec![deep_loop(hierarchy_roots.first().unwrap(), 10_000)];
}

fn deep_seed(prototype: &TraversalSeed, depth: usize) -> TraversalSeed {
    let mut seed = shallow_seed(prototype);
    for _ in 0..depth {
        let mut parent = shallow_seed(prototype);
        parent.children.push(seed);
        seed = parent;
    }
    seed
}

fn shallow_seed(prototype: &TraversalSeed) -> TraversalSeed {
    TraversalSeed {
        polygon: prototype.polygon.clone(),
        depth: prototype.depth,
        is_contour: prototype.is_contour,
        is_smaller_width_perimeter: prototype.is_smaller_width_perimeter,
        extrusion_role: prototype.extrusion_role,
        loop_role: prototype.loop_role,
        route: prototype.route,
        width: prototype.width,
        mm3_per_mm: prototype.mm3_per_mm,
        children: Vec::new(),
    }
}

fn deep_loop(prototype: &PerimeterGeneratorLoop, depth: usize) -> PerimeterGeneratorLoop {
    let mut loop_ = shallow_loop(prototype);
    for _ in 0..depth {
        let mut parent = shallow_loop(prototype);
        parent.children.push(loop_);
        loop_ = parent;
    }
    loop_
}

fn shallow_loop(prototype: &PerimeterGeneratorLoop) -> PerimeterGeneratorLoop {
    PerimeterGeneratorLoop {
        polygon: prototype.polygon.clone(),
        is_contour: prototype.is_contour,
        is_smaller_width_perimeter: prototype.is_smaller_width_perimeter,
        depth: prototype.depth,
        children: Vec::new(),
    }
}
