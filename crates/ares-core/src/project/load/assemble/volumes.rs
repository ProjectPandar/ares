use std::collections::VecDeque;

use super::super::{graph::ModelGraph, mesh_prepare, volume_metadata};
use super::{ExpandedModelBudget, ObjectIdentity, enqueue_occurrence};
use crate::{
    SliceError,
    project::{ProjectVolume, model_settings::PartSettings, transform::Transform3d},
};

pub(super) fn collect_volumes(
    graph: &ModelGraph,
    root: &ObjectIdentity,
    parts: &[PartSettings],
    object_name: &str,
    budget: &mut ExpandedModelBudget,
) -> Result<Vec<ProjectVolume>, SliceError> {
    let mut pending = VecDeque::new();
    enqueue_occurrence(
        &mut pending,
        budget,
        (
            graph.import_namespace(&root.0).to_owned(),
            root.1,
            Transform3d::IDENTITY,
        ),
    )?;
    let mut output = Vec::new();
    let mut unnamed_count = 0_usize;

    while let Some((path, object_id, accumulated)) = pending.pop_front() {
        let Some((model, object)) = graph.imported_object(&path, object_id) else {
            // bbs_3mf.cpp:4867-4891: absent literal importer IDs do not expand.
            continue;
        };
        if let Some(mesh) = &object.mesh {
            if mesh.vertices.vertices.is_empty() || mesh.triangles.triangles.is_empty() {
                continue;
            }
            budget.claim_mesh(mesh.vertices.vertices.len(), mesh.triangles.triangles.len())?;
            let mut selected = volume_metadata::select(parts, output.len(), object.id)?;
            selected.name = volume_name(selected.name, object_name, &mut unnamed_count);
            let prepared = mesh_prepare::prepare(mesh, model.document.unit, accumulated);
            let mut volume = ProjectVolume::new(
                model.path.as_str().to_owned(),
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
            let key = selected
                .mesh_shared_id
                .filter(|key| *key != -1)
                .unwrap_or(object.id as i32);
            volume.set_mesh_preparation(prepared.init_shift, accumulated, key);
            output.push(volume);
        } else if let Some(components) = &object.components {
            for component in &components.components {
                // Main components use their literal p:path; ObjectImporter
                // components use their own namespace (:3851-3872/5564-5584).
                let child_path = component
                    .path
                    .as_deref()
                    .filter(|_| path.is_empty())
                    .unwrap_or(&path)
                    .to_owned();
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
