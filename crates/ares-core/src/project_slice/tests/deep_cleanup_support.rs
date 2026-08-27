//! Shared deep-tree fixtures for constrained-stack cleanup tests.
//!
//! The helpers build 10,000-node traversal and hierarchy chains so cleanup
//! paths can be exercised on the small test stack without recursion overflow.

use crate::project_slice::perimeters::classic::{
    hierarchy::PerimeterGeneratorLoop,
    perimeter_append::PreparedPostClassicPerimeterAppend,
    traversal::{PreparedPostClassicTraversal, TraversalSeed},
};

pub(in crate::project_slice::tests) fn run_on_constrained_stack(
    action: impl FnOnce() + Send + 'static,
) {
    std::thread::Builder::new()
        .stack_size(crate::project_slice::CONSTRAINED_TEST_STACK_SIZE)
        .spawn(action)
        .unwrap()
        .join()
        .unwrap();
}

pub(in crate::project_slice::tests) fn deepen_both_tree_families(
    prepared: &mut PreparedPostClassicTraversal,
) {
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

pub(in crate::project_slice::tests) fn deepen_perimeter_append_trees(
    prepared: &mut PreparedPostClassicPerimeterAppend,
) {
    deepen_both_tree_families(&mut prepared.predecessor);
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
