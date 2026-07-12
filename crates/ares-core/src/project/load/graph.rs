use std::collections::{BTreeMap, BTreeSet, VecDeque};

use crate::SliceError;

use super::super::{
    PackagePath, ProjectArchive,
    content_types::{ContentTypes, MODEL_CONTENT_TYPE, RELATIONSHIPS_CONTENT_TYPE},
    model_xml::{ModelDocument, ModelObject},
    relationships::{MODEL_RELATIONSHIP_TYPE, Relationships},
    xml::{XmlRole, deserialize_xml},
};

pub(super) struct ModelGraph {
    pub models: Vec<LoadedModel>,
    identities: BTreeMap<(PackagePath, u32), (usize, usize)>,
}

pub(super) struct LoadedModel {
    pub path: PackagePath,
    pub document: ModelDocument,
    authorized_targets: BTreeSet<PackagePath>,
}

impl ModelGraph {
    pub fn root(&self) -> &LoadedModel {
        &self.models[0]
    }

    pub fn object(
        &self,
        path: &PackagePath,
        object_id: u32,
    ) -> Result<(&LoadedModel, &ModelObject), SliceError> {
        let &(model_index, object_index) = self
            .identities
            .get(&(path.clone(), object_id))
            .ok_or_else(|| {
                SliceError::InvalidInput(format!(
                    "project component references missing object ({:?}, {object_id})",
                    path.as_str()
                ))
            })?;
        Ok((
            &self.models[model_index],
            &self.models[model_index].document.resources.objects[object_index],
        ))
    }

    pub fn component_target(
        &self,
        owner: &LoadedModel,
        path: Option<&str>,
    ) -> Result<PackagePath, SliceError> {
        let Some(path) = path else {
            return Ok(owner.path.clone());
        };
        let target = owner.path.resolve(path)?;
        if target != owner.path && !owner.authorized_targets.contains(&target) {
            return Err(SliceError::InvalidInput(format!(
                "component path {:?} is not authorized by relationships owned by {:?}",
                target.as_str(),
                owner.path.as_str()
            )));
        }
        Ok(target)
    }
}

pub(super) fn load(
    archive: &mut ProjectArchive<'_>,
    content_types: &ContentTypes,
    archive_paths: &BTreeSet<PackagePath>,
    root: PackagePath,
) -> Result<ModelGraph, SliceError> {
    let mut queue = VecDeque::from([root]);
    let mut loaded_paths = BTreeSet::new();
    let mut models = Vec::new();

    while let Some(path) = queue.pop_front() {
        if !loaded_paths.insert(path.clone()) {
            continue;
        }
        require_model_content_type(content_types, &path)?;
        let bytes = archive.read(&path)?;
        let document: ModelDocument = deserialize_xml(&bytes, XmlRole::Model)?;
        document.validate()?;

        let relationship_part = relationship_part(&path)?;
        let relationships = if archive_paths.contains(&relationship_part) {
            require_relationship_content_type(content_types, &relationship_part)?;
            let bytes = archive.read(&relationship_part)?;
            deserialize_xml::<Relationships>(&bytes, XmlRole::Relationships)?
        } else {
            Relationships {
                relationships: Vec::new(),
            }
        };
        relationships.validate_unique_ids(&path)?;
        let mut authorized_targets = BTreeSet::new();
        for relationship in relationships
            .relationships
            .iter()
            .filter(|entry| entry.relationship_type == MODEL_RELATIONSHIP_TYPE)
        {
            let target = path.resolve(&relationship.target)?;
            require_model_content_type(content_types, &target)?;
            authorized_targets.insert(target.clone());
            queue.push_back(target);
        }

        enqueue_component_targets(&path, &document, &authorized_targets, &mut queue)?;
        models.push(LoadedModel {
            path,
            document,
            authorized_targets,
        });
    }

    validate_archive_ownership(archive_paths, content_types)?;
    let mut identities = BTreeMap::new();
    for (model_index, model) in models.iter().enumerate() {
        for (object_index, object) in model.document.resources.objects.iter().enumerate() {
            if identities
                .insert((model.path.clone(), object.id), (model_index, object_index))
                .is_some()
            {
                return Err(SliceError::InvalidInput(format!(
                    "project repeats object identity ({:?}, {})",
                    model.path.as_str(),
                    object.id
                )));
            }
        }
    }
    let graph = ModelGraph { models, identities };
    validate_component_identities(&graph)?;
    Ok(graph)
}

fn validate_component_identities(graph: &ModelGraph) -> Result<(), SliceError> {
    for model in &graph.models {
        for object in &model.document.resources.objects {
            let Some(components) = &object.components else {
                continue;
            };
            for component in &components.components {
                let target = graph.component_target(model, component.path.as_deref())?;
                graph.object(&target, component.object_id)?;
            }
        }
    }
    Ok(())
}

fn enqueue_component_targets(
    owner: &PackagePath,
    document: &ModelDocument,
    authorized_targets: &BTreeSet<PackagePath>,
    queue: &mut VecDeque<PackagePath>,
) -> Result<(), SliceError> {
    for object in &document.resources.objects {
        let Some(components) = &object.components else {
            continue;
        };
        for component in &components.components {
            let target = match component.path.as_deref() {
                Some(component_path) => owner.resolve(component_path)?,
                None => owner.clone(),
            };
            if target != *owner && !authorized_targets.contains(&target) {
                return Err(SliceError::InvalidInput(format!(
                    "component path {:?} is not authorized by relationships owned by {:?}",
                    target.as_str(),
                    owner.as_str()
                )));
            }
            if target != *owner {
                queue.push_back(target);
            }
        }
    }
    Ok(())
}

fn relationship_part(owner: &PackagePath) -> Result<PackagePath, SliceError> {
    let value = match owner.as_str().rsplit_once('/') {
        Some((directory, file)) => format!("{directory}/_rels/{file}.rels"),
        None => format!("_rels/{}.rels", owner.as_str()),
    };
    PackagePath::entry(value.as_bytes())
}

fn validate_archive_ownership(
    archive_paths: &BTreeSet<PackagePath>,
    content_types: &ContentTypes,
) -> Result<(), SliceError> {
    for path in archive_paths {
        if !path.as_str().ends_with(".model.rels") {
            continue;
        }
        let owner = relationship_owner(path)?;
        if !archive_paths.contains(&owner) {
            return Err(SliceError::InvalidInput(format!(
                "project relationship part {:?} has no owning model",
                path.as_str()
            )));
        }
        require_model_content_type(content_types, &owner)?;
        require_relationship_content_type(content_types, path)?;
    }
    Ok(())
}

fn relationship_owner(path: &PackagePath) -> Result<PackagePath, SliceError> {
    let (directory, file) = if let Some(file) = path.as_str().strip_prefix("_rels/") {
        (None, file)
    } else if let Some((directory, file)) = path.as_str().rsplit_once("/_rels/") {
        (Some(directory), file)
    } else {
        return Err(SliceError::InvalidInput(format!(
            "project relationship part {:?} has no owning model",
            path.as_str()
        )));
    };
    let Some(owner_file) = file
        .strip_suffix(".rels")
        .filter(|file| !file.contains('/'))
    else {
        return Err(SliceError::InvalidInput(format!(
            "project relationship part {:?} has no owning model",
            path.as_str()
        )));
    };
    PackagePath::entry(
        directory
            .map_or_else(
                || owner_file.to_owned(),
                |directory| format!("{directory}/{owner_file}"),
            )
            .as_bytes(),
    )
}

fn require_model_content_type(
    content_types: &ContentTypes,
    path: &PackagePath,
) -> Result<(), SliceError> {
    if content_types.content_type(path) != Some(MODEL_CONTENT_TYPE) {
        return Err(SliceError::InvalidInput(format!(
            "project model part {:?} has the wrong content type",
            path.as_str()
        )));
    }
    Ok(())
}

fn require_relationship_content_type(
    content_types: &ContentTypes,
    path: &PackagePath,
) -> Result<(), SliceError> {
    if content_types.content_type(path) != Some(RELATIONSHIPS_CONTENT_TYPE) {
        return Err(SliceError::InvalidInput(format!(
            "project relationship part {:?} has the wrong content type",
            path.as_str()
        )));
    }
    Ok(())
}
