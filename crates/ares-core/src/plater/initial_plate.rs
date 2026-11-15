use crate::Project;

use super::session::{Model, PlaterSession};

pub(crate) fn initialize_slice_plates(mut project: Project) -> Project {
    if !project.plates().is_empty() {
        return project;
    }
    // PartPlateList::init creates plate 1 before reload_all_objects assigns membership.
    project.initialize_plate(Vec::new());
    let mut session = PlaterSession::with_project(Vec::new(), project);
    session
        .plates
        .reload_all_objects(&Model(&mut session.project));
    let instances = session.plates.plates[0]
        .instances
        .iter()
        .map(|&(object, instance)| {
            let instance = &session.project.objects()[object].instances()[instance];
            [
                instance.object_id(),
                instance.instance_id(),
                instance.labeled_id(),
            ]
        })
        .collect();
    session.project.initialize_plate(instances);
    session.project
}
