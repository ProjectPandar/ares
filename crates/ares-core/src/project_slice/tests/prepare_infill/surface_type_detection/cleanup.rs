use crate::{
    SliceError,
    project_slice::{
        consume_post_surface_type_detection, incomplete_sink,
        perimeters::{
            self,
            classic::{
                hierarchy::PerimeterGeneratorLoop,
                traversal::{PreparedPostClassicTraversal, TraversalSeed},
            },
        },
        prepare_infill::surface_type_detection::{
            self, GeometryStep, fail_geometry_at, geometry_events, reset_geometry_hooks,
            stage_for_test,
        },
    },
};

use super::super::super::support::{KsrArchive, metadata};

const STEPS: [GeometryStep; 19] = [
    GeometryStep::TopSafetyDifference,
    GeometryStep::TopShrink,
    GeometryStep::TopExpand,
    GeometryStep::BottomSafetyDifference,
    GeometryStep::BottomShrink,
    GeometryStep::BottomExpand,
    GeometryStep::CrackIntersection,
    GeometryStep::SingletonCrackErosion,
    GeometryStep::ContainmentDifference,
    GeometryStep::ResidualDifference,
    GeometryStep::CollectionResidualErosion,
    GeometryStep::SingletonCrackExpansion,
    GeometryStep::BottomSubtraction,
    GeometryStep::TopDifference,
    GeometryStep::InternalDifference,
    GeometryStep::FillTopIntersection,
    GeometryStep::FillBottomIntersection,
    GeometryStep::FillBottomBridgeIntersection,
    GeometryStep::FillInternalIntersection,
];

const ERROR: &str = "surface-type detection geometry is outside the supported Clipper range";

#[test]
fn task22o17_success_cleanup_with_both_deep_predecessors_fits_64k_stack() {
    let mut source =
        perimeters::prepare_post_layer_region_perimeters(KsrArchive::new().bytes()).unwrap();
    deepen_both_tree_families(&mut source.predecessor);
    run_on_constrained_stack(move || {
        let output = surface_type_detection::prepare(source).unwrap();
        for object in output.objects {
            incomplete_sink::surface_type_detection::consume_object(object);
        }
        incomplete_sink::consume_boxed_post_classic_traversal(output.predecessor);
    });
}

#[test]
fn task22o17_every_project_staging_failure_with_deep_predecessors_fits_64k_stack() {
    let mut source =
        perimeters::prepare_post_layer_region_perimeters(KsrArchive::new().bytes()).unwrap();
    deepen_both_tree_families(&mut source.predecessor);
    run_on_constrained_stack(move || {
        reset_geometry_hooks();
        stage_for_test(&source).unwrap();
        let reached = geometry_events();
        for step in STEPS {
            assert!(reached.contains(&step), "KSR must reach {step:?}");
        }
        let predecessor = std::ptr::from_ref(source.predecessor.as_ref());
        let allocations = first_record_allocations(&source);
        let (drop_probe, dropped) = source.predecessor.drop_probe_observer();
        for step in STEPS {
            reset_geometry_hooks();
            fail_geometry_at(step);
            assert_eq!(
                stage_for_test(&source),
                Err(SliceError::InvalidInput(ERROR.to_owned())),
                "project staging must fail at {step:?}"
            );
            assert!(geometry_events().contains(&step));
            assert_eq!(std::ptr::from_ref(source.predecessor.as_ref()), predecessor);
            assert_eq!(first_record_allocations(&source), allocations);
        }
        reset_geometry_hooks();
        fail_geometry_at(GeometryStep::FillInternalIntersection);
        let error = match surface_type_detection::prepare(source) {
            Err(error) => error,
            Ok(_) => panic!("late O17 geometry failure must not produce a successor"),
        };
        assert_eq!(error, SliceError::InvalidInput(ERROR.to_owned()));
        assert!(geometry_events().contains(&GeometryStep::FillInternalIntersection));
        assert!(drop_probe.upgrade().is_none());
        assert!(dropped.load(std::sync::atomic::Ordering::SeqCst));
    });
}

#[test]
fn task22o17_preflight_failure_with_deep_predecessors_fits_64k_stack() {
    let mut source =
        perimeters::prepare_post_layer_region_perimeters(KsrArchive::new().bytes()).unwrap();
    deepen_both_tree_families(&mut source.predecessor);
    source.predecessor.resolved.objects[0]
        .object
        .interface_shells = crate::OrcaBool(true);
    let (drop_probe, dropped) = source.predecessor.drop_probe_observer();
    run_on_constrained_stack(move || {
        let error = match surface_type_detection::prepare(source) {
            Err(error) => error,
            Ok(_) => panic!("O17 preflight failure must not produce a successor"),
        };
        assert_eq!(
            error,
            SliceError::UnsupportedProjectFeature("interface_shells".to_owned())
        );
        assert!(drop_probe.upgrade().is_none());
        assert!(dropped.load(std::sync::atomic::Ordering::SeqCst));
    });
}

#[test]
fn task22o17_public_incomplete_cleanup_with_deep_predecessors_fits_64k_stack() {
    let mut source =
        perimeters::prepare_post_layer_region_perimeters(KsrArchive::new().bytes()).unwrap();
    deepen_both_tree_families(&mut source.predecessor);
    let (drop_probe, dropped) = source.predecessor.drop_probe_observer();
    run_on_constrained_stack(move || {
        let output = surface_type_detection::prepare(source).unwrap();
        assert_eq!(
            consume_post_surface_type_detection(output, metadata()).unwrap_err(),
            SliceError::ProjectSlicingIncomplete
        );
        assert!(drop_probe.upgrade().is_none());
        assert!(dropped.load(std::sync::atomic::Ordering::SeqCst));
    });
}

fn first_record_allocations(
    source: &perimeters::layer_region::PreparedPostLayerRegionPerimeters,
) -> [usize; 5] {
    let record = source
        .objects
        .iter()
        .flat_map(|object| object.records.iter().flatten())
        .find(|record| {
            !record.perimeters.is_empty()
                && !record.thin_fills.is_empty()
                && !record.fill_expolygons.is_empty()
                && !record.fill_no_overlap_expolygons.is_empty()
                && !record.fill_surfaces.is_empty()
        })
        .unwrap();
    let gap = match &record.thin_fills[0] {
        perimeters::classic::gap_extrusion::GapFillEntity::Path(path) => {
            path.polyline.points.as_ptr() as usize
        }
        perimeters::classic::gap_extrusion::GapFillEntity::Loop(paths) => {
            paths[0].polyline.points.as_ptr() as usize
        }
    };
    [
        record.perimeters[0].entities.as_ptr() as usize,
        gap,
        record.fill_expolygons[0].contour().points().as_ptr() as usize,
        record.fill_no_overlap_expolygons[0]
            .contour()
            .points()
            .as_ptr() as usize,
        record.fill_surfaces[0]
            .as_parts()
            .1
            .contour()
            .points()
            .as_ptr() as usize,
    ]
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
