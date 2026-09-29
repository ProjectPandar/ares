use super::support::{KsrArchive, metadata};

const PROJECT_SETTINGS: &str = "Metadata/project_settings.config";

/// The G-code of the raft layers (layers 1-4 of a 4-layer raft).
async fn raft_layers(settings: &[(&str, &str)]) -> String {
    let mut archive = KsrArchive::new();
    archive.replace_unique(
        PROJECT_SETTINGS,
        "\"raft_layers\": \"0\"",
        "\"raft_layers\": \"4\"",
    );
    for (from, to) in settings {
        archive.replace_unique(PROJECT_SETTINGS, from, to);
    }
    let output = crate::slice_project(&archive.bytes(), metadata())
        .await
        .unwrap();
    let text = String::from_utf8(output).unwrap();
    text.split("; CHANGE_LAYER\n").skip(1).take(4).collect()
}

/// `SupportParameters.hpp:129-133`: `tree_support_wall_count > 0` turns on
/// the raft base sheath and a honeycomb `support_base_pattern` selects
/// `FillHoneycomb` for the raft base (`SupportCommon.cpp:1476-1482`).
#[tokio::test]
async fn raft_base_follows_sheath_and_honeycomb_settings() {
    let baseline = raft_layers(&[]).await;
    let sheath = raft_layers(&[(
        "\"tree_support_wall_count\": \"0\"",
        "\"tree_support_wall_count\": \"1\"",
    )])
    .await;
    let honeycomb = raft_layers(&[(
        "\"support_base_pattern\": \"default\"",
        "\"support_base_pattern\": \"honeycomb\"",
    )])
    .await;
    assert!(baseline.contains("; FEATURE: Support\n"));
    assert_ne!(sheath, baseline);
    assert_ne!(honeycomb, baseline);
}
