use std::collections::{BTreeMap, BTreeSet, VecDeque};

use crate::{OrcaInt, SliceError};

use super::{
    colors,
    graph::{LoadedModel, ModelGraph},
    mesh_prepare,
    metadata::{self, MetadataIndex},
    volume_metadata,
};
use crate::options::{ObjectOptionOverrides, RegionOptionOverrides};
use crate::project::{
    PackagePath,
    domain::{ProjectInstance, ProjectModel, ProjectObject, ProjectVolume},
    model_settings::{ModelSettings, ObjectSettings, PartSettings},
    transform::Transform3d,
};

#[cfg(test)]
mod tests;

const EXPANDED_MODEL_LIMIT: usize = 1_000_000;
type ObjectIdentity = (PackagePath, u32);
type Pending = (PackagePath, u32, Transform3d);

#[derive(Default)]
struct ExpandedModelBudget {
    used: usize,
}

impl ExpandedModelBudget {
    fn claim(&mut self, count: usize) -> Result<(), SliceError> {
        let used = self
            .used
            .checked_add(count)
            .filter(|used| *used <= EXPANDED_MODEL_LIMIT)
            .ok_or_else(expanded_model_limit_error)?;
        self.used = used;
        Ok(())
    }

    fn claim_mesh(&mut self, vertices: usize, triangles: usize) -> Result<(), SliceError> {
        self.claim(
            vertices
                .checked_add(triangles)
                .ok_or_else(expanded_model_limit_error)?,
        )
    }
}

fn expanded_model_limit_error() -> SliceError {
    SliceError::InvalidInput(
        "project expanded model item count exceeds supported limit of 1000000".to_owned(),
    )
}

fn enqueue_occurrence(
    pending: &mut VecDeque<Pending>,
    budget: &mut ExpandedModelBudget,
    occurrence: Pending,
) -> Result<(), SliceError> {
    budget.claim(1)?;
    pending.push_back(occurrence);
    Ok(())
}

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
    let mut budget = ExpandedModelBudget::default();
    project_domain_with_budget(graph, metadata, settings, &mut budget)
}

fn project_domain_with_budget(
    graph: &ModelGraph,
    metadata: &MetadataIndex,
    settings: &ModelSettings,
    expanded_model_budget: &mut ExpandedModelBudget,
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
    let build_roots = validate_bare_metadata_identity(graph, root)?;
    validate_component_cycles(graph, &build_roots)?;
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
                    &identity,
                    source_settings.map_or(&[], |object| object.parts.as_slice()),
                    &object_metadata.0,
                    expanded_model_budget,
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
) -> Result<Vec<ObjectIdentity>, SliceError> {
    let mut paths = BTreeMap::<u32, PackagePath>::new();
    let mut root_set = BTreeSet::new();
    let mut roots = Vec::new();
    for item in &root.document.build.items {
        let target_path = graph.component_target(root, item.path.as_deref())?;
        graph.object(&target_path, item.object_id)?;

        let identity = (target_path.clone(), item.object_id);
        if root_set.insert(identity.clone()) {
            roots.push(identity);
        }

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
    Ok(roots)
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum VisitColor {
    Gray,
    Black,
}

fn next_component(
    graph: &ModelGraph,
    frame: &mut (PackagePath, u32, usize),
) -> Result<Option<ObjectIdentity>, SliceError> {
    let (path, object_id, component_index) = frame;
    let (model, object) = graph.object(path, *object_id)?;
    let Some(component) = object
        .components
        .as_ref()
        .and_then(|components| components.components.get(*component_index))
    else {
        return Ok(None);
    };
    *component_index += 1;
    Ok(Some((
        graph.component_target(model, component.path.as_deref())?,
        component.object_id,
    )))
}

fn validate_component_cycles(
    graph: &ModelGraph,
    roots: &[ObjectIdentity],
) -> Result<(), SliceError> {
    let mut colors = BTreeMap::<ObjectIdentity, VisitColor>::new();
    for root in roots {
        if colors.contains_key(root) {
            continue;
        }
        colors.insert(root.clone(), VisitColor::Gray);
        let mut stack = vec![(root.0.clone(), root.1, 0_usize)];

        while !stack.is_empty() {
            let child = next_component(graph, stack.last_mut().unwrap())?;

            let Some(child) = child else {
                let (path, object_id, _) = stack.pop().unwrap();
                colors.insert((path, object_id), VisitColor::Black);
                continue;
            };
            match colors.get(&child).copied() {
                Some(VisitColor::Gray) => {
                    return Err(invalid("component graph contains a cycle"));
                }
                Some(VisitColor::Black) => {}
                None => {
                    colors.insert(child.clone(), VisitColor::Gray);
                    stack.push((child.0, child.1, 0));
                }
            }
        }
    }
    Ok(())
}

fn collect_volumes(
    graph: &ModelGraph,
    root: &ObjectIdentity,
    parts: &[PartSettings],
    object_name: &str,
    budget: &mut ExpandedModelBudget,
) -> Result<Vec<ProjectVolume>, SliceError> {
    let mut pending = VecDeque::<Pending>::new();
    enqueue_occurrence(
        &mut pending,
        budget,
        (root.0.clone(), root.1, Transform3d::IDENTITY),
    )?;
    let mut output = Vec::new();
    let mut unnamed_count = 0_usize;

    while let Some((path, object_id, accumulated)) = pending.pop_front() {
        let (model, object) = graph.object(&path, object_id)?;
        if let Some(mesh) = &object.mesh {
            if mesh.vertices.vertices.is_empty() || mesh.triangles.triangles.is_empty() {
                continue;
            }
            budget.claim_mesh(mesh.vertices.vertices.len(), mesh.triangles.triangles.len())?;
            let mut selected = volume_metadata::select(parts, output.len(), object.id)?;
            selected.name = volume_name(selected.name, object_name, &mut unnamed_count);
            let prepared = mesh_prepare::prepare(mesh, model.document.unit, accumulated);
            let mut volume = ProjectVolume::new(
                path.as_str().to_owned(),
                object.id,
                prepared.mesh,
                prepared.transform,
                (
                    selected.name,
                    selected.volume_type,
                    selected.region_overrides,
                    selected.source_transform,
                ),
            );
            volume.set_mesh_shared(selected.mesh_shared);
            output.push(volume);
        } else if let Some(components) = &object.components {
            for component in &components.components {
                let child_path = graph.component_target(model, component.path.as_deref())?;
                enqueue_occurrence(
                    &mut pending,
                    budget,
                    (
                        child_path,
                        component.object_id,
                        accumulated.then(component.transform),
                    ),
                )?;
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

fn invalid(reason: impl std::fmt::Display) -> SliceError {
    SliceError::InvalidInput(format!("invalid project model graph: {reason}"))
}
