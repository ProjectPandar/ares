use crate::{
    SliceError,
    geometry::{Point, Polygon},
    project_slice::{
        incomplete_sink,
        perimeters::{
            classic::{
                materialize,
                traversal::{PendingPathBranch, PreparedTraversalSurface, TraversalSeed},
            },
            prepare_post_classic_traversal,
        },
    },
};

use super::super::super::super::support::ksr_project;

#[test]
fn task22o7_deep_materialization_and_error_cleanup_fit_a_constrained_stack() {
    let ordinary = prepare_post_classic_traversal(ksr_project()).unwrap();
    let overhang = prepare_post_classic_traversal(ksr_project()).unwrap();
    std::thread::Builder::new()
        .stack_size(64 * 1024)
        .spawn(move || {
            materialize_deep_ordinary(ordinary);
            reject_after_deep_overhang(overhang);
        })
        .unwrap()
        .join()
        .unwrap();
}

fn materialize_deep_ordinary(
    mut prepared: Box<crate::project_slice::perimeters::classic::PreparedPostClassicTraversal>,
) {
    let surface = surface_for_branch(&mut prepared, false);
    let prototype = surface.roots.first().unwrap();
    surface.roots = vec![deep_seed(prototype, 10_000)];

    let raw = materialize::finish(prepared).unwrap();
    for object in raw.objects {
        incomplete_sink::consume_raw_path_object(object);
    }
    incomplete_sink::consume_boxed_post_classic_traversal(raw.predecessor);
}

fn reject_after_deep_overhang(
    mut prepared: Box<crate::project_slice::perimeters::classic::PreparedPostClassicTraversal>,
) {
    let surface = surface_for_branch(&mut prepared, true);
    let prototype = surface.roots.first().unwrap();
    let deep = deep_seed(prototype, 2_000);
    let mut invalid = shallow_seed(prototype);
    let high = 0x4000_0000_0000_0000_i64;
    invalid.polygon = Polygon::new(vec![
        Point::new(high, 0),
        Point::new(high + 10, 0),
        Point::new(high + 10, 10),
    ]);
    surface.roots = vec![deep, invalid];

    assert!(matches!(
        materialize::finish(prepared),
        Err(SliceError::InvalidInput(message))
            if message == "classic perimeter raw path coordinate is outside the supported Clipper range"
    ));
}

fn surface_for_branch(
    prepared: &mut crate::project_slice::perimeters::classic::PreparedPostClassicTraversal,
    overhang: bool,
) -> &mut PreparedTraversalSurface {
    prepared
        .objects
        .iter_mut()
        .flat_map(|object| object.records.iter_mut().flatten())
        .filter(|record| {
            matches!(record.branch, PendingPathBranch::OverhangClipping { .. }) == overhang
        })
        .flat_map(|record| record.surfaces.iter_mut())
        .find(|surface| !surface.roots.is_empty())
        .unwrap()
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
