//! bbs_3mf.cpp:4931-5052; Model.cpp:1295-1307/2821-2824.
use super::{Point3d, ProjectMesh, ProjectObject, Transform3d, mesh::MeshGeometry};
use std::{collections::BTreeMap, sync::Arc};

#[derive(Clone, Debug, PartialEq)]
pub(super) struct MeshPreparation {
    geometry: Arc<MeshGeometry>,
    pub(super) init_shift: Point3d,
    pub(super) component: Transform3d,
    pub(super) key: i32,
}

impl MeshPreparation {
    pub(super) fn new(mesh: &ProjectMesh, component: Transform3d, key: i32) -> Self {
        Self {
            geometry: mesh.geometry.clone(),
            init_shift: Point3d::new(0.0, 0.0, 0.0),
            component,
            key,
        }
    }
}

pub(crate) fn materialize_meshes(objects: &mut [ProjectObject]) {
    // Importer m_objects is an ordered map; output/build order is independent.
    let mut order = (0..objects.len()).collect::<Vec<_>>();
    order.sort_by(|&a, &b| {
        (objects[a].source_model_path(), objects[a].id())
            .cmp(&(objects[b].source_model_path(), objects[b].id()))
    });
    let mut owners = BTreeMap::new();
    for index in order {
        for volume in &mut objects[index].volumes {
            let source = &volume.mesh_preparation;
            match owners.entry(source.key) {
                std::collections::btree_map::Entry::Vacant(entry) => {
                    entry.insert((source.geometry.clone(), source.init_shift));
                    volume.mesh.geometry = source.geometry.clone();
                    volume.transform = source.component.translated(source.init_shift);
                }
                std::collections::btree_map::Entry::Occupied(entry) => {
                    let (geometry, shift) = entry.get();
                    volume.mesh.geometry = geometry.clone();
                    // Shared shift is added AFTER component transformation,
                    // unlike fresh C * Translation(s); preserve current facets.
                    volume.transform = source.component.pretranslated(*shift);
                }
            }
        }
    }
}
