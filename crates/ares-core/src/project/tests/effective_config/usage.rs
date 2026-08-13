mod raw_support;
mod roles;
mod wipe;

use crate::{
    LayerConfigRange, ObjectOptions, OrcaInt, Percent, Point3d, ProcessBrimType, ProjectInstance,
    ProjectMesh, ProjectObject, ProjectSettings, ProjectVolume, ProjectVolumeType, RegionOptions,
    load_project,
    options::{ObjectOptionOverrides, RegionOptionOverrides},
    project::{
        effective_config::{
            ValidatedMaterializedProject,
            candidates::resolve_project_candidates,
            grouping::group_print_object_transforms,
            types::{
                BoundedProjectUsage, ProjectUsageCoverage, ResolvedLayerCandidate,
                ResolvedModelPartCandidate, ResolvedPrintObjectConfig, ResolvedProjectObject,
            },
            usage::{ProjectUsageSources, collect_bounded_project_usage},
        },
        transform::Transform3d,
    },
};

use super::{support::ProjectParts, valid_settings};

#[test]
fn bounded_usage_exposes_typed_coverage() {
    let source_settings = settings(2);
    let usage = collect(&source_settings, &source_settings, &[], &[]);

    let _: &[usize] = &usage.supported_used_filaments;
    match usage.coverage {
        ProjectUsageCoverage::TypedConfigSourcesOnly => {}
    }
}

fn settings(logical_count: usize) -> ProjectSettings {
    let mut settings = valid_settings(logical_count + 1, logical_count);
    settings.process.object.brim_type = ProcessBrimType::NoBrim;
    settings
}

fn validated(logical_count: usize) -> ValidatedMaterializedProject {
    ValidatedMaterializedProject {
        physical_extruder_count: logical_count + 1,
        logical_filament_count: logical_count,
    }
}

fn collect(
    source_settings: &ProjectSettings,
    wipe_settings: &ProjectSettings,
    objects: &[ProjectObject],
    resolved: &[ResolvedProjectObject],
) -> BoundedProjectUsage {
    let grouped = group_print_object_transforms(objects);
    collect_bounded_project_usage(
        ProjectUsageSources {
            settings: source_settings,
            objects,
            grouped: &grouped,
            resolved,
        },
        wipe_settings,
    )
}

fn resolved_object(object: ObjectOptions, regions: Vec<RegionOptions>) -> ResolvedProjectObject {
    let model_parts = regions
        .into_iter()
        .enumerate()
        .map(|(volume_index, region)| ResolvedModelPartCandidate {
            volume_index,
            region,
        })
        .collect();
    ResolvedProjectObject {
        source_object_index: 0,
        object,
        print_objects: vec![ResolvedPrintObjectConfig {
            transform: Transform3d::IDENTITY,
        }],
        layer_candidates: vec![ResolvedLayerCandidate {
            min_z: 0.0,
            max_z: f64::MAX,
            source_range_index: None,
            model_parts,
        }],
    }
}

fn object_options(settings: &ProjectSettings) -> ObjectOptions {
    let mut options = ObjectOptions::from_base(&settings.process.object);
    options.brim_type = ProcessBrimType::NoBrim;
    options
}

fn base_region(settings: &ProjectSettings) -> RegionOptions {
    let mut region = RegionOptions::from_base(&settings.process.region);
    region.wall_loops = OrcaInt(0);
    region.sparse_infill_density = Percent(0.0);
    region.top_shell_layers = OrcaInt(0);
    region.bottom_shell_layers = OrcaInt(0);
    region.outer_wall_filament_id = OrcaInt(1);
    region.inner_wall_filament_id = OrcaInt(1);
    region.sparse_infill_filament_id = OrcaInt(1);
    region.internal_solid_filament_id = OrcaInt(1);
    region.top_surface_filament_id = OrcaInt(1);
    region.bottom_surface_filament_id = OrcaInt(1);
    region
}

fn printable_source() -> ProjectObject {
    source_object(
        Default::default(),
        Default::default(),
        Vec::new(),
        vec![Transform3d::IDENTITY],
        Vec::new(),
    )
}

fn source_object(
    object_overrides: ObjectOptionOverrides,
    region_overrides: RegionOptionOverrides,
    volumes: Vec<ProjectVolume>,
    transforms: Vec<Transform3d>,
    ranges: Vec<LayerConfigRange>,
) -> ProjectObject {
    let instances = transforms
        .into_iter()
        .enumerate()
        .map(|(index, transform)| {
            ProjectInstance::new(
                [
                    1,
                    u32::try_from(index).unwrap(),
                    1_000 + u32::try_from(index).unwrap(),
                ],
                true,
                false,
                transform,
            )
        })
        .collect();
    let mut object = ProjectObject::new(
        "synthetic.model".to_owned(),
        1,
        (
            "object".to_owned(),
            String::new(),
            object_overrides,
            region_overrides,
        ),
        volumes,
        instances,
    );
    object.set_layer_config_ranges(ranges);
    object
}

fn volume(
    volume_type: ProjectVolumeType,
    extruder: Option<i32>,
    z: f64,
    nonempty: bool,
) -> ProjectVolume {
    let vertices = vec![
        Point3d::new(0.0, 0.0, z),
        Point3d::new(1.0, 0.0, z),
        Point3d::new(0.0, 1.0, z),
    ];
    let triangles = if nonempty {
        vec![[0, 1, 2]]
    } else {
        Vec::new()
    };
    ProjectVolume::new(
        "synthetic.model".to_owned(),
        1,
        ProjectMesh::new(vertices, triangles),
        Transform3d::IDENTITY,
        (
            "volume".to_owned(),
            volume_type,
            RegionOptionOverrides {
                extruder: extruder.map(OrcaInt),
                ..Default::default()
            },
            Transform3d::IDENTITY,
        ),
    )
}

fn layer_ranges(body: &str) -> Vec<LayerConfigRange> {
    let mut parts = ProjectParts::valid();
    parts.insert_text(
        "Metadata/layer_config_ranges.xml",
        &format!(r#"<objects><object id="1">{body}</object></objects>"#),
    );
    load_project(parts.bytes()).unwrap().objects()[0]
        .layer_config_ranges()
        .to_vec()
}

fn resolve_candidates(
    settings: &ProjectSettings,
    logical_count: usize,
    objects: &[ProjectObject],
) -> Vec<ResolvedProjectObject> {
    let grouped = group_print_object_transforms(objects);
    resolve_project_candidates(settings, validated(logical_count), objects, &grouped).unwrap()
}

fn z_translation(z: f64) -> Transform3d {
    Transform3d::parse_3mf(&format!("1 0 0 0 1 0 0 0 1 0 0 {z}")).unwrap()
}
