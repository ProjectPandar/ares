//! Selected importer object membership (`bbs_3mf.cpp:2058-2082`).
use super::{PlateMetadata, Project, ProjectObject};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum ImportScope {
    All,
    Plate(u32),
}

impl Project {
    pub(super) fn selected_import_objects(&self, selected: &PlateMetadata) -> Vec<ProjectObject> {
        if self.import_scope == ImportScope::Plate(selected.id()) {
            return self.objects.clone();
        }
        let mut objects = self
            .objects
            .iter()
            .filter(|object| {
                selected
                    .instances()
                    .iter()
                    .any(|identity| identity[0] == object.id())
            })
            .cloned()
            .collect::<Vec<_>>();
        for object in &mut objects {
            object.instances.clone_from(&object.import_instances);
        }
        super::materialize_meshes(&mut objects);
        super::normalize_volume_offsets(&mut objects);
        objects
    }

    /// Scope the loaded model before profile placement; retain full grid metadata.
    pub(crate) fn prepare_import_scope(&mut self, plate: Option<u32>) {
        // Invalid plate errors stay at the existing final selection boundary;
        // do not move them ahead of profile/config validation.
        let Some(selected) = plate.and_then(|id| self.plates.iter().find(|p| p.id() == id)) else {
            return;
        };
        let id = selected.id();
        self.objects = self.selected_import_objects(selected);
        self.import_scope = ImportScope::Plate(id);
    }
}
