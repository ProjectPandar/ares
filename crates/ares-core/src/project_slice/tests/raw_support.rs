use crate::{
    Point3d, ProjectMesh, ProjectObject, ProjectVolume, ProjectVolumeType, SliceError, Transform3d,
    geometry::CoordinateScale, options::RegionOptionOverrides,
    project::effective_config::types::ResolvedProjectObject,
};

use super::super::{
    layers::{PlannedLayer, PlannedPrintObject},
    raw_intersections::{
        IntersectedPrintObject, ProjectedPrintObject, intersect_projected_objects,
    },
};
use super::support::{
    identity_resolved, object, object_with_instances, plan, project_volume, project_volume_at_x,
    project_with_range, resolved_object, slot_limit, transform, unsupported,
};

pub(super) fn retained_facts(
    projected: &ProjectedPrintObject,
    source: &ProjectObject,
) -> Vec<(u32, u32, ProjectVolumeType)> {
    projected
        .volumes()
        .iter()
        .map(|volume| {
            let source_volume = &source.volumes()[volume.source_volume_index()];
            (
                volume.ordinal(),
                source_volume.id(),
                source_volume.volume_type(),
            )
        })
        .collect()
}

pub(super) fn intersections(
    objects: &[ProjectObject],
    resolved: &[ResolvedProjectObject],
    plans: Vec<PlannedPrintObject>,
) -> Result<Vec<IntersectedPrintObject>, SliceError> {
    let projected = super::support::project(objects, resolved, plans)?;
    intersect_projected_objects(objects, resolved, projected, CoordinateScale::Normal)
}

pub(super) fn planned_layers(
    source_object_index: usize,
    transform_index: usize,
    layers: &[(f64, f64)],
) -> PlannedPrintObject {
    PlannedPrintObject {
        source_object_index,
        transform_index,
        layers: layers
            .iter()
            .enumerate()
            .map(|(id, &(print_z, slice_z))| PlannedLayer {
                id,
                height: 0.2,
                print_z,
                slice_z,
            })
            .collect(),
    }
}

pub(super) fn mesh_volume(
    id: u32,
    volume_type: ProjectVolumeType,
    vertices: Vec<Point3d>,
    triangles: Vec<[u32; 3]>,
    transform: Transform3d,
) -> ProjectVolume {
    ProjectVolume::new(
        "raw-transform.model".to_owned(),
        id,
        ProjectMesh::new(vertices, triangles),
        transform,
        (
            format!("volume-{id}"),
            volume_type,
            RegionOptionOverrides::default(),
            Transform3d::IDENTITY,
        ),
    )
}

pub(super) fn ordinal_gap_object() -> ProjectObject {
    use ProjectVolumeType::{
        ModelPart, NegativeVolume, ParameterModifier, SupportBlocker, SupportEnforcer,
    };

    object(
        "root.model",
        10,
        vec![
            project_volume("root.model", 9, ModelPart, false, false),
            project_volume_at_x("root.model", 8, SupportBlocker, 1_000.0),
            project_volume("root.model", 7, ModelPart, true, false),
            project_volume("root.model", 6, ParameterModifier, true, false),
            project_volume_at_x("root.model", 5, SupportEnforcer, 1_000.0),
            project_volume("root.model", 4, NegativeVolume, true, false),
        ],
        &[Transform3d::IDENTITY],
    )
}

pub(super) fn bfs_restart_request() -> (
    Vec<ProjectObject>,
    Vec<ResolvedProjectObject>,
    Vec<PlannedPrintObject>,
) {
    let first = object(
        "first.model",
        10,
        [3, 1, 2]
            .map(|id| project_volume("first.model", id, ProjectVolumeType::ModelPart, true, false))
            .into(),
        &[
            Transform3d::IDENTITY,
            transform("1 0 0 0 1 0 0 0 1 19 -7 0"),
        ],
    );
    let second = object(
        "second.model",
        20,
        vec![project_volume(
            "second.model",
            99,
            ProjectVolumeType::NegativeVolume,
            true,
            false,
        )],
        &[Transform3d::IDENTITY],
    );
    (
        vec![first, second],
        vec![identity_resolved(0), identity_resolved(1)],
        vec![plan(0, 0, 1), plan(1, 0, 1)],
    )
}

pub(super) fn unique_unprinted_shared_request() -> (
    Vec<ProjectObject>,
    Vec<ResolvedProjectObject>,
    Vec<PlannedPrintObject>,
) {
    let printed = object(
        "printed.model",
        30,
        vec![project_volume(
            "printed-leaf.model",
            80,
            ProjectVolumeType::ModelPart,
            true,
            false,
        )],
        &[Transform3d::IDENTITY],
    );
    let unprinted_shared = object_with_instances(
        "unprinted.model",
        40,
        vec![project_volume(
            "unprinted-leaf.model",
            81,
            ProjectVolumeType::SupportEnforcer,
            true,
            true,
        )],
        &[(false, Transform3d::IDENTITY)],
    );
    (
        vec![printed, unprinted_shared],
        vec![identity_resolved(0)],
        vec![plan(0, 0, 1)],
    )
}

pub(super) fn ranged_later_request() -> (
    Vec<ProjectObject>,
    Vec<ResolvedProjectObject>,
    Vec<PlannedPrintObject>,
) {
    let invalid_geometry = object(
        "invalid-first.model",
        1,
        vec![project_volume_at_x(
            "invalid-first.model",
            91,
            ProjectVolumeType::ModelPart,
            f64::from(f32::MAX),
        )],
        &[Transform3d::IDENTITY],
    );
    let mut ranged = object(
        "ranged-later.model",
        2,
        vec![project_volume(
            "ranged-later.model",
            92,
            ProjectVolumeType::ModelPart,
            true,
            false,
        )],
        &[Transform3d::IDENTITY],
    );
    ranged.set_layer_config_ranges(
        project_with_range(0.0, 1.0, 1).objects()[0]
            .layer_config_ranges()
            .to_vec(),
    );
    (
        vec![invalid_geometry, ranged],
        vec![identity_resolved(0), identity_resolved(1)],
        vec![plan(0, 0, 1), plan(1, 0, 1)],
    )
}

pub(super) fn exact_dense_object() -> ProjectObject {
    use ProjectVolumeType::{ModelPart, SupportBlocker, SupportEnforcer};

    let mut volumes = (100..110)
        .map(|id| project_volume("exact.model", id, ModelPart, true, false))
        .collect::<Vec<_>>();
    volumes.extend([
        project_volume("exact.model", 110, ModelPart, false, false),
        project_volume("exact.model", 111, SupportBlocker, true, false),
        project_volume("exact.model", 112, SupportEnforcer, true, false),
    ]);
    object(
        "exact.model",
        10,
        volumes,
        &[
            Transform3d::IDENTITY,
            transform("1 0 0 0 1 0 0 0 1 19 -7 0"),
        ],
    )
}

pub(super) fn request_wide_dense_objects() -> Vec<ProjectObject> {
    [10_usize, 11]
        .into_iter()
        .enumerate()
        .map(|(object_index, count)| {
            let path = format!("request-{object_index}.model");
            object(
                &path,
                object_index as u32 + 20,
                (0..count)
                    .map(|volume_index| {
                        project_volume(
                            &path,
                            200 + (object_index * 20 + volume_index) as u32,
                            ProjectVolumeType::ModelPart,
                            true,
                            false,
                        )
                    })
                    .collect(),
                &[Transform3d::IDENTITY],
            )
        })
        .collect()
}

type PreflightScenario = (
    Vec<ProjectObject>,
    Vec<ResolvedProjectObject>,
    Vec<PlannedPrintObject>,
    SliceError,
);

pub(super) fn preflight_order_scenarios() -> [PreflightScenario; 4] {
    let distinct = transform("1 0 0 0 1 0 0 0 1 0 0 2");

    let mut range_later = object(
        "range-later.model",
        2,
        vec![project_volume(
            "range-later.model",
            250,
            ProjectVolumeType::ModelPart,
            true,
            false,
        )],
        &[Transform3d::IDENTITY],
    );
    range_later.set_layer_config_ranges(
        project_with_range(0.0, 1.0, 1).objects()[0]
            .layer_config_ranges()
            .to_vec(),
    );
    let range = (
        vec![dense_object(1, false), range_later],
        vec![
            resolved_object(0, &[Transform3d::IDENTITY, distinct]),
            identity_resolved(1),
        ],
        vec![plan(0, 0, 100_000), plan(1, 0, 1)],
        unsupported("layer_config_ranges"),
    );

    let sharing_first = object(
        "sharing-first.model",
        3,
        vec![project_volume(
            "sharing-first.model",
            350,
            ProjectVolumeType::ModelPart,
            true,
            true,
        )],
        &[Transform3d::IDENTITY],
    );
    let centering_later = object(
        "centering-later.model",
        4,
        vec![project_volume(
            "centering-later.model",
            450,
            ProjectVolumeType::ModelPart,
            true,
            false,
        )],
        &[Transform3d::IDENTITY],
    );
    let centering = (
        vec![sharing_first, centering_later],
        vec![
            identity_resolved(0),
            resolved_object(1, &[Transform3d::IDENTITY, distinct]),
        ],
        vec![plan(0, 0, 1), plan(1, 0, 1)],
        unsupported("print_object_centering"),
    );

    let sharing_later = object(
        "sharing-later.model",
        6,
        vec![project_volume(
            "sharing-later.model",
            650,
            ProjectVolumeType::ModelPart,
            true,
            true,
        )],
        &[Transform3d::IDENTITY],
    );
    let sharing = (
        vec![dense_object(5, false), sharing_later],
        vec![identity_resolved(0), identity_resolved(1)],
        vec![plan(0, 0, 100_000), plan(1, 0, 1)],
        unsupported("shared_mesh_centering"),
    );
    let dense = (
        vec![dense_object(7, true)],
        vec![identity_resolved(0)],
        vec![plan(0, 0, 100_000)],
        slot_limit(),
    );
    [range, centering, sharing, dense]
}

fn dense_object(object_id: u32, large_coordinate: bool) -> ProjectObject {
    let path = format!("dense-{object_id}.model");
    let mut volumes = (0..11)
        .map(|id| {
            project_volume(
                &path,
                object_id * 100 + id,
                ProjectVolumeType::ModelPart,
                true,
                false,
            )
        })
        .collect::<Vec<_>>();
    if large_coordinate {
        volumes[0] = project_volume_at_x(
            &path,
            object_id * 100,
            ProjectVolumeType::ModelPart,
            f64::from(f32::MAX),
        );
    }
    object(&path, object_id, volumes, &[Transform3d::IDENTITY])
}
