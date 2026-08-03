use crate::project_slice::{
    incomplete_sink,
    perimeters::{
        classic::{
            hierarchy::PerimeterGeneratorLoop,
            infill_boundary::{self, GeometryStep},
            traversal::{PreparedPostClassicTraversal, TraversalSeed},
        },
        prepare_post_classic_gap_extrusion, prepare_post_classic_infill_boundary,
    },
};

use super::super::super::super::support::{KsrArchive, ksr_project};

const CONFIG: &str = "Metadata/project_settings.config";

#[test]
fn task22o15_success_and_numeric_cleanup_fit_constrained_stack() {
    let mut success = prepare_post_classic_infill_boundary(ksr_project()).unwrap();
    deepen_both_tree_families(&mut success.predecessor);
    run_on_constrained_stack(move || {
        for object in success.objects {
            incomplete_sink::consume_infill_boundary_object(object);
        }
        incomplete_sink::consume_boxed_post_classic_traversal(success.predecessor);
    });

    let mut archive = KsrArchive::new();
    archive.replace_unique(
        CONFIG,
        "\"top_bottom_infill_wall_overlap\": \"25%\"",
        "\"top_bottom_infill_wall_overlap\": \"1e308%\"",
    );
    let mut numeric = prepare_post_classic_gap_extrusion(archive.bytes()).unwrap();
    deepen_both_tree_families(&mut numeric.predecessor);
    run_on_constrained_stack(move || {
        assert!(infill_boundary::finish(numeric).is_err());
    });
}

macro_rules! deep_geometry_cleanup_test {
    ($name:ident, $step:expr) => {
        #[test]
        fn $name() {
            let mut source = source_for_step($step);
            deepen_both_tree_families(&mut source.predecessor);
            run_on_constrained_stack(move || {
                infill_boundary::reset_geometry_hooks();
                infill_boundary::fail_geometry_at($step);
                assert!(infill_boundary::finish(source).is_err());
            });
        }
    };
}

deep_geometry_cleanup_test!(
    task22o15_deep_simplify_error_cleanup,
    GeometryStep::Simplify
);
deep_geometry_cleanup_test!(
    task22o15_deep_aggregate_union_error_cleanup,
    GeometryStep::AggregateUnion
);
deep_geometry_cleanup_test!(
    task22o15_deep_ordinary_offset_error_cleanup,
    GeometryStep::OrdinaryOffset
);
deep_geometry_cleanup_test!(
    task22o15_deep_top_offset_error_cleanup,
    GeometryStep::TopOffset
);
deep_geometry_cleanup_test!(
    task22o15_deep_top_intersection_error_cleanup,
    GeometryStep::TopIntersection
);
deep_geometry_cleanup_test!(
    task22o15_deep_top_overlap_offset_error_cleanup,
    GeometryStep::TopOverlapOffset
);
deep_geometry_cleanup_test!(
    task22o15_deep_top_union_error_cleanup,
    GeometryStep::TopUnion
);
deep_geometry_cleanup_test!(
    task22o15_deep_no_overlap_two_error_cleanup,
    GeometryStep::NoOverlapTwo
);
deep_geometry_cleanup_test!(
    task22o15_deep_no_overlap_one_error_cleanup,
    GeometryStep::NoOverlapOne
);
deep_geometry_cleanup_test!(
    task22o15_deep_final_top_union_error_cleanup,
    GeometryStep::FinalTopUnion
);

fn source_for_step(
    step: GeometryStep,
) -> crate::project_slice::perimeters::classic::gap_extrusion::PreparedPostClassicGapExtrusion {
    if step == GeometryStep::NoOverlapOne {
        let mut archive = KsrArchive::new();
        archive.replace_unique(
            CONFIG,
            "\"infill_wall_overlap\": \"15%\"",
            "\"infill_wall_overlap\": \"100%\"",
        );
        prepare_post_classic_gap_extrusion(archive.bytes()).unwrap()
    } else {
        prepare_post_classic_gap_extrusion(ksr_project()).unwrap()
    }
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
