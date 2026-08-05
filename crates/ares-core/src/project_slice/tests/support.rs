mod archive;

pub(super) use archive::KsrArchive;

use super::super::{
    layers::{PlannedLayer, PlannedPrintObject},
    raw_intersections::{ProjectedPrintObject, prepare_projected_objects},
};
use crate::{
    GenerationMetadata, ObjectOptions, OrcaInt, Point3d, ProjectInstance, ProjectMesh,
    ProjectObject, ProjectSettings, ProjectVolume, ProjectVolumeType, RegionOptions, SliceError,
    Transform3d, load_project,
    options::{ObjectOptionOverrides, RegionOptionOverrides},
    project::effective_config::types::{
        ResolvedLayerCandidate, ResolvedModelPartCandidate, ResolvedPrintObjectConfig,
        ResolvedProjectObject,
    },
};

const KSR_PROJECT: &[u8] =
    include_bytes!("../../../../../tests/ksr_fdmtest_v4/ksr_fdmtest_v4.project.3mf");
pub(super) fn ksr_project() -> &'static [u8] {
    KSR_PROJECT
}

pub(super) fn metadata() -> GenerationMetadata {
    GenerationMetadata::deterministic(2026, 7, 16, 1, 2, 3)
}

pub(super) fn object_options() -> ObjectOptions {
    ObjectOptions::from_base(&ProjectSettings::default().process.object)
}

pub(super) fn region() -> RegionOptions {
    RegionOptions::from_base(&ProjectSettings::default().process.region)
}

pub(super) fn source(
    object_extruder: Option<i32>,
    volumes: &[(ProjectVolumeType, Option<i32>)],
) -> ProjectObject {
    let object_region = RegionOptionOverrides {
        extruder: object_extruder.map(OrcaInt),
        ..Default::default()
    };
    ProjectObject::new(
        "synthetic.model".to_owned(),
        1,
        (
            "object".to_owned(),
            String::new(),
            ObjectOptionOverrides::default(),
            object_region,
        ),
        volumes
            .iter()
            .enumerate()
            .map(|(index, (volume_type, extruder))| {
                let region = RegionOptionOverrides {
                    extruder: extruder.map(OrcaInt),
                    ..Default::default()
                };
                ProjectVolume::new(
                    "synthetic.model".to_owned(),
                    index as u32 + 1,
                    ProjectMesh::new(vec![Point3d::new(0.0, 0.0, 1.0)], Vec::new()),
                    Transform3d::IDENTITY,
                    (
                        "volume".to_owned(),
                        *volume_type,
                        region,
                        Transform3d::IDENTITY,
                    ),
                )
            })
            .collect(),
        vec![ProjectInstance::new(
            [1, 0, 1_000],
            true,
            false,
            Transform3d::IDENTITY,
        )],
    )
}

pub(super) fn project_volume(
    source_model_path: &str,
    id: u32,
    volume_type: ProjectVolumeType,
    nonempty: bool,
    mesh_shared: bool,
) -> ProjectVolume {
    project_volume_fixture(
        source_model_path,
        id,
        volume_type,
        (nonempty, mesh_shared, 0.0),
    )
}

pub(super) fn project_volume_at_x(
    source_model_path: &str,
    id: u32,
    volume_type: ProjectVolumeType,
    x: f64,
) -> ProjectVolume {
    project_volume_fixture(source_model_path, id, volume_type, (true, false, x))
}

fn project_volume_fixture(
    source_model_path: &str,
    id: u32,
    volume_type: ProjectVolumeType,
    fixture: (bool, bool, f64),
) -> ProjectVolume {
    let (nonempty, mesh_shared, x) = fixture;
    let mesh = if nonempty {
        ProjectMesh::new(
            vec![
                Point3d::new(x, 0.0, 0.0),
                Point3d::new(x, 1.0, 0.0),
                Point3d::new(x, 0.0, 1.0),
            ],
            vec![[0, 1, 2]],
        )
    } else {
        ProjectMesh::new(Vec::new(), Vec::new())
    };
    let mut volume = ProjectVolume::new(
        source_model_path.to_owned(),
        id,
        mesh,
        Transform3d::IDENTITY,
        (
            format!("volume-{id}"),
            volume_type,
            RegionOptionOverrides::default(),
            Transform3d::IDENTITY,
        ),
    );
    volume.set_mesh_shared(mesh_shared);
    volume
}

pub(super) fn object(
    source_model_path: &str,
    id: u32,
    volumes: Vec<ProjectVolume>,
    instance_transforms: &[Transform3d],
) -> ProjectObject {
    let instances = instance_transforms
        .iter()
        .copied()
        .map(|transform| (true, transform))
        .collect::<Vec<_>>();
    object_with_instances(source_model_path, id, volumes, &instances)
}

pub(super) fn object_with_instances(
    source_model_path: &str,
    id: u32,
    volumes: Vec<ProjectVolume>,
    instances: &[(bool, Transform3d)],
) -> ProjectObject {
    ProjectObject::new(
        source_model_path.to_owned(),
        id,
        (
            format!("object-{id}"),
            String::new(),
            ObjectOptionOverrides::default(),
            RegionOptionOverrides::default(),
        ),
        volumes,
        instances
            .iter()
            .copied()
            .enumerate()
            .map(|(index, (printable, transform))| {
                ProjectInstance::new(
                    [id, index as u32, 1_000 + index as u32],
                    printable,
                    false,
                    transform,
                )
            })
            .collect(),
    )
}

pub(super) fn resolved_object(
    source_object_index: usize,
    transforms: &[Transform3d],
) -> ResolvedProjectObject {
    ResolvedProjectObject {
        source_object_index,
        object: ObjectOptions::from_base(&ProjectSettings::default().process.object),
        print_objects: transforms
            .iter()
            .copied()
            .map(|transform| ResolvedPrintObjectConfig { transform })
            .collect(),
        layer_candidates: vec![ResolvedLayerCandidate {
            min_z: 0.0,
            max_z: 1.0,
            source_range_index: None,
            model_parts: Vec::new(),
        }],
    }
}

pub(super) fn plan(
    source_object_index: usize,
    transform_index: usize,
    layer_count: usize,
) -> PlannedPrintObject {
    PlannedPrintObject {
        source_object_index,
        transform_index,
        layers: vec![
            PlannedLayer {
                id: 0,
                height: 0.2,
                print_z: 0.2,
                slice_z: 0.1,
            };
            layer_count
        ],
    }
}

pub(super) fn project(
    objects: &[ProjectObject],
    resolved: &[ResolvedProjectObject],
    plans: Vec<PlannedPrintObject>,
) -> Result<Vec<ProjectedPrintObject>, SliceError> {
    prepare_projected_objects(objects, resolved, plans)
}

pub(super) fn identity_resolved(source_object_index: usize) -> ResolvedProjectObject {
    resolved_object(source_object_index, &[Transform3d::IDENTITY])
}

pub(super) fn transform(value: &str) -> Transform3d {
    Transform3d::parse_3mf(value).unwrap()
}

pub(super) fn unsupported(feature: &str) -> SliceError {
    SliceError::UnsupportedProjectFeature(feature.to_owned())
}

pub(super) fn slot_limit() -> SliceError {
    SliceError::InvalidInput(
        "project raw intersection layer slot count exceeds supported limit of 1000000".to_owned(),
    )
}

pub(super) fn resolved(
    source_object_index: usize,
    object: ObjectOptions,
    regions: Vec<RegionOptions>,
) -> ResolvedProjectObject {
    ResolvedProjectObject {
        source_object_index,
        object,
        print_objects: vec![ResolvedPrintObjectConfig {
            transform: Transform3d::IDENTITY,
        }],
        layer_candidates: vec![ResolvedLayerCandidate {
            min_z: 0.0,
            max_z: 1.0,
            source_range_index: None,
            model_parts: regions
                .into_iter()
                .enumerate()
                .map(|(volume_index, region)| ResolvedModelPartCandidate {
                    volume_index,
                    region,
                })
                .collect(),
        }],
    }
}

pub(super) fn project_with_range(min_z: f64, max_z: f64, extruder: i32) -> crate::Project {
    let mut archive = KsrArchive::new();
    archive.insert_text(
        "Metadata/layer_config_ranges.xml",
        &format!(
            r#"<objects><object id="1"><range min_z="{min_z}" max_z="{max_z}"><option opt_key="extruder">{extruder}</option></range></object></objects>"#
        ),
    );
    load_project(archive.bytes()).unwrap()
}
