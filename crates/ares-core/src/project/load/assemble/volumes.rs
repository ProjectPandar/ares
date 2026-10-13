use std::collections::VecDeque;

use super::super::{graph::ModelGraph, mesh_prepare, volume_metadata};
use super::{ExpandedModelBudget, ObjectIdentity, Pending, enqueue_occurrence};
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
            let key = selected
                .mesh_shared_id
                .filter(|key| *key != -1)
                .unwrap_or(object.id as i32);
            volume.set_mesh_preparation(prepared.init_shift, accumulated, key);
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
