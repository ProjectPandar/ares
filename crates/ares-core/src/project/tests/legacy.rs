//! `PrintConfigDef::handle_legacy` (`PrintConfig.cpp:8033`) runs inside
//! `set_deserialize` (`Config.cpp:577-579`) before each option is stored, so a
//! loaded `DynamicPrintConfig` — and therefore the flat config `Slic3r::validate`
//! checks (`OrcaSlicer.cpp:3647`) — is already post-legacy. These tests pin the
//! same post-legacy semantics on `ProjectSettingsRaw` so the O3 range loop reads
//! the normalized view (e.g. `ironing_angle: -1` -> `0`) rather than the source
//! token.

use super::model::fixture::ProjectParts;
use crate::load_project;

fn raw_token(project: &crate::Project, key: &str) -> Option<String> {
    project
        .documents()
        .project_settings_raw
        .iter()
        .find_map(|entry| {
            let (name, token) = entry.expect("raw token renders");
            (name == key).then_some(token)
        })
        .flatten()
}

/// `ironing_angle` negative inputs normalize to `"0"`
/// (`PrintConfig.cpp:8181-8182`: `opt_key == "ironing_angle" &&
/// boost::starts_with(value, "-")` → `value = "0"`), so a `-1` source value is
/// stored as `0` and never reaches the `[0,359]` range check.
#[test]
fn project_settings_raw_normalizes_negative_ironing_angle() {
    let mut parts = ProjectParts::valid();
    parts.insert_text(
        "Metadata/project_settings.config",
        r#"{"ironing_angle":"-1","layer_height":"0.2"}"#,
    );
    let project = load_project(parts.bytes()).unwrap();
    assert_eq!(raw_token(&project, "ironing_angle"), Some("0".to_owned()));
}
