mod archive;
mod content_types;
mod domain;
mod filament_sequence;
mod load;
mod model_settings;
mod model_xml;
mod plate;
mod relationships;
mod slice_info;
mod transform;
mod xml;
mod xml_characters;

pub(crate) use archive::{ArchiveLimits, PackagePath, ProjectArchive};
pub use domain::{
    PlateMetadata, Point3d, Project, ProjectInstance, ProjectMesh, ProjectModel, ProjectObject,
    ProjectVolume,
};
pub use load::load_project;
pub use transform::Transform3d;

#[cfg(test)]
mod tests {
    mod archive;
    mod documents;
    mod model;
    mod path;
    mod transform;
    mod xml_limits;
}
