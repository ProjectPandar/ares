use super::super::support::KsrArchive;

const PROJECT_SETTINGS: &str = "Metadata/project_settings.config";
const RESOLUTION: &str = r#""resolution": "0.012""#;

pub(super) fn resolution(value: &str) -> Vec<u8> {
    let mut archive = KsrArchive::new();
    archive.replace_unique(
        PROJECT_SETTINGS,
        RESOLUTION,
        &format!(r#""resolution": "{value}""#),
    );
    archive.bytes()
}
