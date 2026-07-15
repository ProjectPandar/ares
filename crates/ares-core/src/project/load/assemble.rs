use std::collections::{BTreeMap, BTreeSet, VecDeque};

use crate::{OrcaInt, SliceError};

use super::{
    colors,
    graph::{LoadedModel, ModelGraph},
    metadata::{self, MetadataIndex},
    volume_metadata,
};
use crate::options::{ObjectOptionOverrides, RegionOptionOverrides};
use crate::project::{
    PackagePath,
    domain::{Point3d, ProjectInstance, ProjectMesh, ProjectModel, ProjectObject, ProjectVolume},
    model_settings::{ModelSettings, ObjectSettings, PartSettings},
    model_xml::Mesh,
    transform::Transform3d,
};

struct ObjectBuilder {
    metadata: (String, String, ObjectOptionOverrides, RegionOptionOverrides),
    volumes: Vec<ProjectVolume>,
    instances: Vec<ProjectInstance>,
}

pub(super) fn project_domain(
    graph: &ModelGraph,
    metadata: &MetadataIndex,
    settings: &ModelSettings,
) -> Result<(Vec<ProjectModel>, Vec<ProjectObject>), SliceError> {
    let models = graph
        .models
        .iter()
        .map(|model| {
            ProjectModel::new(
                model.path.as_str().to_owned(),
                model
                    .document
                    .resources
                    .objects
                    .iter()
                    .map(|object| object.id)
                    .collect(),
            )
        })
        .collect();

    let root = graph.root();
    validate_bare_metadata_identity(graph, root)?;
    let group_extruders = colors::group_extruders(graph);

    let mut builders = BTreeMap::<(PackagePath, u32), ObjectBuilder>::new();
    let mut order = Vec::new();
    let mut loaded_instances = BTreeSet::new();
    for item in &root.document.build.items {
        let target_path = graph.component_target(root, item.path.as_deref())?;
        let (_, source_object) = graph.object(&target_path, item.object_id)?;
        let identity = (target_path.clone(), item.object_id);
        let instance_id = builders
            .get(&identity)
            .map_or(0, |builder| builder.instances.len() as u32);
        let loaded_label_id = *metadata
            .loaded_labels
            .get(&(item.object_id, instance_id))
            .ok_or_else(|| {
                invalid(format!(
                    "build instance ({}, {instance_id}) has no loaded plate identity",
                    item.object_id
                ))
            })?;

        if let std::collections::btree_map::Entry::Vacant(entry) = builders.entry(identity.clone())
        {
            let source_settings = metadata
                .object_settings
                .get(&item.object_id)
                .map(|index| &settings.objects[*index]);
            let object_ordinal = order.len() + 1;
            let object_metadata = object_metadata(
                source_settings,
                &source_object.name,
                source_object.pid,
                object_ordinal,
                &group_extruders,
            );
            entry.insert(ObjectBuilder {
                volumes: collect_volumes(
                    graph,
                    &target_path,
                    item.object_id,
                    source_settings.map_or(&[], |object| object.parts.as_slice()),
                    &object_metadata.0,
                )?,
                metadata: object_metadata,
                instances: Vec::new(),
            });
            order.push(identity.clone());
        }
        builders
            .get_mut(&identity)
            .unwrap()
            .instances
            .push(ProjectInstance::new(
                [item.object_id, instance_id, loaded_label_id],
                item.printable,
                item.auto_drop,
                item.transform,
            ));
        loaded_instances.insert((item.object_id, instance_id));
    }

    if metadata
        .loaded_labels
        .keys()
        .copied()
        .collect::<BTreeSet<_>>()
        != loaded_instances
    {
        return Err(invalid(
            "plate metadata references an object or instance absent from the build",
        ));
    }
    metadata::validate_assemble(settings, &loaded_instances)?;

    let objects = order
        .into_iter()
        .map(|identity| {
            let builder = builders.remove(&identity).unwrap();
            ProjectObject::new(
                identity.0.as_str().to_owned(),
                identity.1,
                builder.metadata,
                builder.volumes,
                builder.instances,
            )
        })
        .collect();
    Ok((models, objects))
}

fn object_metadata(
    settings: Option<&ObjectSettings>,
    xml_name: &str,
    pid: i32,
    ordinal: usize,
    group_extruders: &BTreeMap<i32, i32>,
) -> (String, String, ObjectOptionOverrides, RegionOptionOverrides) {
    if let Some(settings) = settings {
        return (
            settings.name.clone(),
            settings.module.clone(),
            settings.overrides.clone(),
            settings.region_overrides.clone(),
        );
    }

    let name = if xml_name.is_empty() {
        format!("Object_{ordinal}")
    } else {
        xml_name.to_owned()
    };
    let region_overrides = RegionOptionOverrides {
        extruder: group_extruders.get(&pid).copied().map(OrcaInt),
        ..Default::default()
    };
    (
        name,
        String::new(),
        ObjectOptionOverrides::default(),
        region_overrides,
    )
}

fn validate_bare_metadata_identity(
    graph: &ModelGraph,
    root: &LoadedModel,
) -> Result<(), SliceError> {
    let mut paths = BTreeMap::<u32, PackagePath>::new();
    for item in &root.document.build.items {
        let target_path = graph.component_target(root, item.path.as_deref())?;
        graph.object(&target_path, item.object_id)?;

        match paths.entry(item.object_id) {
            std::collections::btree_map::Entry::Vacant(entry) => {
                entry.insert(target_path);
            }
            std::collections::btree_map::Entry::Occupied(entry) if entry.get() != &target_path => {
                return Err(invalid(format!(
                    "ambiguous object ID {} maps to both {} and {} at the bare model-settings/plate/assemble metadata boundary",
                    item.object_id,
                    entry.get().as_str(),
                    target_path.as_str()
                )));
            }
            std::collections::btree_map::Entry::Occupied(_) => {}
        }
    }
    Ok(())
}

fn collect_volumes(
    graph: &ModelGraph,
    path: &PackagePath,
    object_id: u32,
    parts: &[PartSettings],
    object_name: &str,
) -> Result<Vec<ProjectVolume>, SliceError> {
    type Pending = (PackagePath, u32, Transform3d, Vec<(PackagePath, u32)>);
    let mut pending =
        VecDeque::<Pending>::from([(path.clone(), object_id, Transform3d::IDENTITY, Vec::new())]);
    let mut output = Vec::new();
    let mut unnamed_count = 0_usize;

    while let Some((path, object_id, accumulated, mut ancestors)) = pending.pop_front() {
        let identity = (path.clone(), object_id);
        if ancestors.contains(&identity) {
            return Err(invalid("component graph contains a cycle"));
        }
        ancestors.push(identity);
        let (model, object) = graph.object(&path, object_id)?;
        if let Some(mesh) = &object.mesh {
            let mut selected = volume_metadata::select(parts, output.len(), object.id)?;
            selected.name = volume_name(selected.name, object_name, &mut unnamed_count);
            output.push(ProjectVolume::new(
                path.as_str().to_owned(),
                object.id,
                project_mesh(mesh, model),
                accumulated,
                (
                    selected.name,
                    selected.volume_type,
                    selected.region_overrides,
                    selected.source_transform,
                ),
            ));
        } else if let Some(components) = &object.components {
            for component in &components.components {
                let child_path = graph.component_target(model, component.path.as_deref())?;
                pending.push_back((
                    child_path,
                    component.object_id,
                    accumulated.then(component.transform),
                    ancestors.clone(),
                ));
            }
        }
    }
    Ok(output)
}

fn volume_name(name: String, object_name: &str, unnamed_count: &mut usize) -> String {
    if !name.is_empty() {
        return name;
    }
    *unnamed_count += 1;
    match *unnamed_count {
        1 => object_name.to_owned(),
        count => format!("{object_name}_{count}"),
    }
}

fn project_mesh(mesh: &Mesh, model: &LoadedModel) -> ProjectMesh {
    let factor = model.document.unit.millimeter_factor();
    ProjectMesh::new(
        mesh.vertices
            .vertices
            .iter()
            .map(|vertex| Point3d {
                x: vertex.x * factor,
                y: vertex.y * factor,
                z: vertex.z * factor,
            })
            .collect(),
        mesh.triangles
            .triangles
            .iter()
            .map(|triangle| [triangle.v1, triangle.v2, triangle.v3])
            .collect(),
    )
}

fn invalid(reason: impl std::fmt::Display) -> SliceError {
    SliceError::InvalidInput(format!("invalid project model graph: {reason}"))
}
