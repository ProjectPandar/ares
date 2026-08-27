mod archive;
mod content_types;
mod domain;
pub(crate) mod effective_config;
mod filament_sequence;
mod layer_config_ranges;
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
    ProjectVolume, ProjectVolumeType,
};
pub use layer_config_ranges::LayerConfigRange;
pub use load::load_project;
pub use transform::Transform3d;

#[cfg(test)]
mod tests {
    mod archive;
    mod documents;
    mod effective_config;
    mod layer_config_ranges;
    mod model;
    mod orca_cli_export;
    mod path;
    mod task22b_transform;
    mod transform;
    mod xml_limits;
}
