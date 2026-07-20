use crate::{
    Point3d, ProjectMesh, ProjectVolume, ProjectVolumeType, Transform3d,
    geometry::{ExPolygon, Point, Polygon},
    mesh_slicer::SlicingMode,
    options::RegionOptionOverrides,
};

pub(in crate::project_slice::tests) mod complex;

use super::{
    super::{
        closing::{PostClosingLayer, PostClosingPrintObject, PostClosingVolume},
        layers::{PlannedLayer, PlannedPrintObject},
        region_slices::{PendingRegionSlices, RegionSurfaceKind, prepare_region_slices},
        volume_bounds::{BoundedPrintObject, build_volume_bounds},
        volume_regions::{VolumeRegion, VolumeRegionGraph},
    },
    support::{object, region, resolved_object},
};

#[test]
fn task22j_region_slices_clone_complete_sidecar_and_allocate_dense_internal_layers() {
    let first = shape(0);
    let second = shape(20);
    let pending = compose(
        &[(0, 0.2, 0.1), (1, 0.4, 0.3), (2, 0.6, 0.5)],
        vec![volume_case(
            7,
            ProjectVolumeType::ModelPart,
            [0.0, 0.0, 0.0, 10.0, 10.0, 1.0],
            vec![vec![first.clone(), second.clone()], Vec::new(), Vec::new()],
        )],
        2,
        &[(0, ProjectVolumeType::ModelPart, Some(0))],
    );

    let (output, working, _, _, complex) = pending.as_parts();
    let (plan, sidecar, regions) = output.as_parts();
    assert_eq!(plan.layers.len(), 3);
    assert!(complex.is_empty());
    assert_eq!(sidecar.len(), 1);
    let (occurrence, sidecar_layers) = sidecar[0].as_parts();
    assert_eq!(occurrence.get(), 7);
    assert_eq!(
        sidecar_layers,
        &[vec![first.clone(), second.clone()], vec![], vec![]]
    );
    assert_eq!(working[0].as_parts().1, &[vec![], vec![], vec![]]);
    assert_eq!(regions.len(), 2);
    for region in regions {
        assert_eq!(region.as_parts().2.len(), 3);
    }
    let surfaces = regions[0].as_parts().2[0].surfaces();
    assert_eq!(surfaces.len(), 2);
    for (surface, expected) in surfaces.iter().zip([first, second]) {
        let (kind, expolygon, thickness, layers, bridge, extra) = surface.as_parts();
        assert_eq!(kind, RegionSurfaceKind::Internal);
        assert_eq!(kind as u8, 4);
        assert_eq!(expolygon, &expected);
        assert_eq!((thickness, layers, bridge, extra), (-1.0, 1, -1.0, 0));
    }
    assert!(regions[0].as_parts().2[1].surfaces().is_empty());
    assert!(regions[0].as_parts().2[2].surfaces().is_empty());
    assert!(
        regions[1]
            .as_parts()
            .2
            .iter()
            .all(|layer| layer.surfaces().is_empty())
    );
    let (owned, _, _, _, _) = pending.into_parts();
    assert_eq!(owned.as_parts().0.layers.len(), 3);
}

#[test]
fn task22j_region_slices_zero_and_single_record_fast_table_is_exact() {
    for (kind, moves) in [
        (None, false),
        (Some(ProjectVolumeType::ModelPart), true),
        (Some(ProjectVolumeType::NegativeVolume), false),
        (Some(ProjectVolumeType::ParameterModifier), false),
    ] {
        let cases = kind
            .map(|kind| {
                vec![volume_case(
                    1,
                    kind,
                    [0.0, 0.0, 0.0, 1.0, 1.0, 1.0],
                    vec![vec![shape(100)]],
                )]
            })
            .unwrap_or_default();
        let records = kind
            .map(|kind| vec![(0, kind, (kind == ProjectVolumeType::ModelPart).then_some(0))])
            .unwrap_or_default();
        let pending = compose(&[(0, 0.2, 0.1)], cases, 1, &records);
        let (output, working, _, _, complex) = pending.as_parts();
        let surfaces = output.as_parts().2[0].as_parts().2[0].surfaces();
        assert_eq!(surfaces.len(), usize::from(moves));
        assert!(complex.is_empty());
        if let Some(volume) = working.first() {
            assert_eq!(volume.as_parts().1[0].len(), usize::from(!moves));
        }
    }
}

#[test]
#[rustfmt::skip]
fn task22j_region_slices_sort_physical_occurrences_but_use_source_graph_priority() {
    let first = shape(200);
    let later = shape(300);
    let pending = compose(
        &[(0, 0.2, 0.1)],
        vec![
            volume_case(90, ProjectVolumeType::ModelPart, [0.0, 0.0, 0.0, 1.0, 1.0, 1.0], vec![vec![first.clone()]]),
            volume_case(10, ProjectVolumeType::ModelPart, [10.0, 10.0, 0.0, 11.0, 11.0, 1.0], vec![vec![later.clone()]]),
        ],
        2,
        &[(0, ProjectVolumeType::ModelPart, Some(0)), (1, ProjectVolumeType::ModelPart, Some(1))],
    );

    let (output, working, _, records, complex) = pending.as_parts();
    assert_eq!(
        output
            .as_parts()
            .1
            .iter()
            .map(|volume| volume.as_parts().0.get())
            .collect::<Vec<_>>(),
        vec![10, 90]
    );
    assert_eq!(
        records
            .iter()
            .map(|record| record.occurrence_id.get())
            .collect::<Vec<_>>(),
        vec![90, 10]
    );
    assert!(complex.is_empty());
    let regions = output.as_parts().2;
    assert_eq!(regions[0].as_parts().2[0].surfaces()[0].as_parts().1, &first);
    assert!(regions[1].as_parts().2[0].surfaces().is_empty());
    assert_eq!(working[0].as_parts().1[0], vec![later]);
    assert!(working[1].as_parts().1[0].is_empty());
}

#[test]
#[rustfmt::skip]
fn task22j_region_slices_first_active_model_uses_inclusive_slice_z_f32_not_print_z() {
    let inactive = shape(400);
    let active = shape(500);
    let slice_z = f64::from(1.0_f32 + 1e-4_f32);
    let pending = compose(
        &[(0, 99.0, slice_z)],
        vec![
            volume_case(1, ProjectVolumeType::ModelPart, [0.0, 0.0, 2.0, 1.0, 1.0, 3.0], vec![vec![inactive.clone()]]),
            volume_case(2, ProjectVolumeType::ModelPart, [10.0, 10.0, 1.0, 11.0, 11.0, 1.0], vec![vec![active.clone()]]),
        ],
        2,
        &[(0, ProjectVolumeType::ModelPart, Some(0)), (1, ProjectVolumeType::ModelPart, Some(1))],
    );

    let (output, working, _, _, complex) = pending.as_parts();
    let regions = output.as_parts().2;
    assert!(complex.is_empty());
    assert!(regions[0].as_parts().2[0].surfaces().is_empty());
    assert_eq!(regions[1].as_parts().2[0].surfaces()[0].as_parts().1, &active);
    assert_eq!(working[0].as_parts().1[0], vec![inactive]);
    assert!(working[1].as_parts().1[0].is_empty());
}

#[test]
#[rustfmt::skip]
fn task22j_region_slices_touching_xy_collects_ordered_complex_layers_and_retains_working() {
    let pending = compose(
        &[(0, 9.0, 0.1), (1, 9.5, 1.0), (2, 10.0, 2.0)],
        vec![
            volume_case(1, ProjectVolumeType::ModelPart, [0.0, 0.0, 0.0, 1.0, 1.0, 3.0], vec![vec![shape(600)], vec![shape(610)], vec![shape(620)]]),
            volume_case(2, ProjectVolumeType::NegativeVolume, [1.0, 0.25, 0.0, 2.0, 0.75, 0.2], vec![vec![shape(700)], vec![], vec![]]),
            volume_case(3, ProjectVolumeType::NegativeVolume, [0.25, 1.0, 1.9, 0.75, 2.0, 2.1], vec![vec![], vec![], vec![shape(800)]]),
        ],
        1,
        &[
            (0, ProjectVolumeType::ModelPart, Some(0)),
            (1, ProjectVolumeType::NegativeVolume, None),
            (2, ProjectVolumeType::NegativeVolume, None),
        ],
    );

    let (output, working, _, _, complex) = pending.as_parts();
    assert_eq!(complex, &[0, 2]);
    let layers = output.as_parts().2[0].as_parts().2;
    assert!(layers[0].surfaces().is_empty());
    assert_eq!(layers[1].surfaces().len(), 1);
    assert!(layers[2].surfaces().is_empty());
    assert_eq!(working[0].as_parts().1.iter().map(Vec::len).collect::<Vec<_>>(), vec![1, 0, 1]);
    assert_eq!(working[1].as_parts().1.iter().map(Vec::len).collect::<Vec<_>>(), vec![1, 0, 0]);
    assert_eq!(working[2].as_parts().1.iter().map(Vec::len).collect::<Vec<_>>(), vec![0, 0, 1]);
}

#[test]
#[rustfmt::skip]
fn task22j_region_slices_no_active_model_keeps_dense_output_empty_and_working_geometry_intact() {
    let pending = compose(
        &[(0, 99.0, 5.0)],
        vec![
            volume_case(1, ProjectVolumeType::ModelPart, [0.0, 0.0, 0.0, 1.0, 1.0, 1.0], vec![vec![shape(900)]]),
            volume_case(2, ProjectVolumeType::ModelPart, [2.0, 2.0, 2.0, 3.0, 3.0, 3.0], vec![vec![shape(910)]]),
        ],
        2,
        &[(0, ProjectVolumeType::ModelPart, Some(0)), (1, ProjectVolumeType::ModelPart, Some(1))],
    );
    let (output, working, _, _, complex) = pending.as_parts();
    assert!(output.as_parts().2.iter().all(|region| region.as_parts().2[0].surfaces().is_empty()));
    assert!(complex.is_empty());
    assert_eq!(working.iter().map(|volume| volume.as_parts().1[0].len()).collect::<Vec<_>>(), vec![1, 1]);
}

struct VolumeCase {
    ordinal: u32,
    kind: ProjectVolumeType,
    bbox: [f64; 6],
    layers: Vec<Vec<ExPolygon>>,
}

fn volume_case(
    ordinal: u32,
    kind: ProjectVolumeType,
    bbox: [f64; 6],
    layers: Vec<Vec<ExPolygon>>,
) -> VolumeCase {
    VolumeCase {
        ordinal,
        kind,
        bbox,
        layers,
    }
}

fn compose(
    layer_zs: &[(usize, f64, f64)],
    cases: Vec<VolumeCase>,
    region_count: usize,
    records: &[(usize, ProjectVolumeType, Option<usize>)],
) -> PendingRegionSlices {
    let bounded = bounded(layer_zs, cases);
    let graph = graph(&bounded, region_count, records);
    prepare_region_slices(bounded, graph)
}

fn bounded(layer_zs: &[(usize, f64, f64)], cases: Vec<VolumeCase>) -> BoundedPrintObject {
    let source_volumes = cases.iter().map(source_volume).collect();
    let source = object(
        "synthetic.model",
        1,
        source_volumes,
        &[Transform3d::IDENTITY],
    );
    let post = PostClosingPrintObject::new(
        PlannedPrintObject {
            source_object_index: 0,
            transform_index: 0,
            layers: layer_zs
                .iter()
                .map(|&(id, print_z, slice_z)| PlannedLayer {
                    id,
                    height: 0.2,
                    print_z,
                    slice_z,
                })
                .collect(),
        },
        cases
            .into_iter()
            .enumerate()
            .map(|(source, case)| {
                PostClosingVolume::new(
                    source,
                    case.ordinal,
                    case.kind,
                    case.layers
                        .into_iter()
                        .map(|expolygons| PostClosingLayer::new(SlicingMode::Regular, expolygons))
                        .collect(),
                )
            })
            .collect(),
    );
    build_volume_bounds(&source, &resolved_object(0, &[Transform3d::IDENTITY]), post)
}

fn source_volume(case: &VolumeCase) -> ProjectVolume {
    let [min_x, min_y, min_z, max_x, max_y, max_z] = case.bbox;
    ProjectVolume::new(
        "synthetic.model".to_owned(),
        case.ordinal + 1_000,
        ProjectMesh::new(
            vec![
                Point3d::new(min_x, min_y, min_z),
                Point3d::new(max_x, min_y, max_z),
                Point3d::new(min_x, max_y, min_z),
            ],
            vec![[0, 1, 2]],
        ),
        Transform3d::IDENTITY,
        (
            format!("volume-{}", case.ordinal),
            case.kind,
            RegionOptionOverrides::default(),
            Transform3d::IDENTITY,
        ),
    )
}

fn graph(
    bounded: &BoundedPrintObject,
    region_count: usize,
    records: &[(usize, ProjectVolumeType, Option<usize>)],
) -> VolumeRegionGraph {
    VolumeRegionGraph {
        all_regions: (0..region_count).map(|_| region()).collect(),
        volume_regions: records
            .iter()
            .map(|&(source, kind, region_id)| VolumeRegion {
                source_volume_index: source,
                occurrence_id: bounded
                    .volume_for_source_index(source)
                    .unwrap()
                    .occurrence_id(),
                kind,
                parent: None,
                region_id,
                bound_index: bounded.bound_index_for_source_index(source).unwrap(),
            })
            .collect(),
    }
}

fn shape(x: i64) -> ExPolygon {
    ExPolygon::new(
        Polygon::new(vec![
            Point::new(x, 0),
            Point::new(x + 10, 0),
            Point::new(x + 10, 10),
            Point::new(x, 10),
        ]),
        Vec::new(),
    )
}
