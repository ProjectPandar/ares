use crate::project::{ArchiveLimits, PackagePath, ProjectArchive};


pub(super) fn project_settings_bytes() -> Vec<u8> {
    let mut archive = ProjectArchive::open(crate::test_fixtures::ksr_project(), ArchiveLimits::PROJECT).unwrap();
    archive
        .read(&PackagePath::entry(b"Metadata/project_settings.config").unwrap())
        .unwrap()
}

pub(super) fn project_settings_value() -> serde_json::Value {
    serde_json::from_slice(&project_settings_bytes()).unwrap()
}
