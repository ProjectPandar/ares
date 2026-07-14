use crate::project::{ArchiveLimits, PackagePath, ProjectArchive};

const FIXTURE: &[u8] = include_bytes!(
    "../../../../../tests/ksr_fdmtest_v4/ksr_fdmtest_v4.project.3mf"
);

pub(super) fn project_settings_bytes() -> Vec<u8> {
    let mut archive = ProjectArchive::open(FIXTURE, ArchiveLimits::PROJECT).unwrap();
    archive
        .read(&PackagePath::entry(b"Metadata/project_settings.config").unwrap())
        .unwrap()
}

pub(super) fn project_settings_value() -> serde_json::Value {
    serde_json::from_slice(&project_settings_bytes()).unwrap()
}
