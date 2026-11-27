use std::collections::{BTreeMap, VecDeque};

use crate::{OrcaInt, SliceError};

use super::{colors, graph::ModelGraph, metadata::MetadataIndex};
use crate::options::{ObjectOptionOverrides, RegionOptionOverrides};
use crate::project::{
    PackagePath,
    domain::{
        ProjectInstance, ProjectModel, ProjectObject, ProjectVolume, object_id::ObjectIdCounter,
    },
    model_settings::{ModelSettings, ObjectSettings},
};

mod component_graph;
mod volumes;
use component_graph::{validate_bare_metadata_identity, validate_component_cycles};
use volumes::collect_volumes;

const EXPANDED_MODEL_LIMIT: usize = 1_000_000;
type ObjectIdentity = (PackagePath, u32);

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

fn enqueue_occurrence<T>(
    pending: &mut VecDeque<T>,
    budget: &mut ExpandedModelBudget,
    occurrence: T,
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

    let mut object_ids = ObjectIdCounter::model_read();
    let mut builders = BTreeMap::<ObjectIdentity, ObjectBuilder>::new();
    let mut order = Vec::new();
    for item in &root.document.build.items {
        let target_path = graph.component_target(root, item.path.as_deref())?;
        let (_, source_object) = graph.object(&target_path, item.object_id)?;
        let identity = (target_path.clone(), item.object_id);
        let instance_id = builders
            .get(&identity)
            .map_or(0, |builder| builder.instances.len() as u32);
        let saved_label = metadata.saved_labels.get(&(item.object_id, instance_id));

        if let std::collections::btree_map::Entry::Vacant(entry) = builders.entry(identity.clone())
        {
            object_ids.add_object();
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
        // bbs_3mf.cpp:4381–4448 creates build instances before deferred volumes (:5212).
        let allocated_label_id = object_ids.add_instance();
        let instance = match saved_label {
            Some(label) if label.loaded => ProjectInstance::new(
                [item.object_id, instance_id, label.identify_id],
                item.printable,
                item.auto_drop,
                item.transform,
            ),
            _ => ProjectInstance::unassigned(
                [item.object_id, instance_id, allocated_label_id],
                item.printable,
                item.auto_drop,
                item.transform,
                saved_label.map(|label| label.identify_id),
            ),
        };
        builders
            .get_mut(&identity)
            .unwrap()
            .instances
            .push(instance);
    }

    if let Some(assemble) = settings
        .assemble
        .as_ref()
        .filter(|doc| !doc.items.is_empty())
    {
        let build_ids: BTreeMap<_, _> = build_roots.iter().map(|root| (root.1, root)).collect();
        for item in &assemble.items {
            let Some(identity) = u32::try_from(item.object_id)
                .ok()
                .and_then(|id| build_ids.get(&id))
            else {
                return Err(invalid(format!(
                    "can not find object for assemble item, id= {}",
                    item.object_id
                )));
            };
            // bbs_3mf.cpp:5025–5047: absent objects fail; absent instances are ignored.
            if let Ok(ordinal) = usize::try_from(item.instance_id)
                && let Some(instance) = builders
                    .get_mut(*identity)
                    .unwrap()
                    .instances
                    .get_mut(ordinal)
            {
                instance.set_assemble(item.state());
            }
        }
    }

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

fn invalid(reason: impl std::fmt::Display) -> SliceError {
    SliceError::InvalidInput(format!("invalid project model graph: {reason}"))
}
