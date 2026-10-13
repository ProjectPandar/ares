use std::collections::{BTreeMap, BTreeSet};

use super::super::graph::{LoadedModel, ModelGraph};
use super::{ObjectIdentity, invalid};
use crate::{SliceError, project::PackagePath};

pub(super) fn validate_bare_metadata_identity(
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

pub(super) fn validate_component_cycles(
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
