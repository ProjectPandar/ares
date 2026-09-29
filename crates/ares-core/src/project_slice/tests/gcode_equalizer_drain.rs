use super::support::{KsrArchive, metadata};

const PROJECT_SETTINGS: &str = "Metadata/project_settings.config";

/// With `max_volumetric_extrusion_rate_slope > 0` the PressureEqualizer
/// holds each layer back until the next one (`PressureEqualizer.cpp:
/// 174-200`); the trailing NOP layer result (`GCode.cpp:3752`) must still
/// drain the final layer into the output.
#[tokio::test]
async fn pressure_equalizer_keeps_the_final_layer() {
    let mut archive = KsrArchive::new();
    archive.replace_unique(
        PROJECT_SETTINGS,
        "\"max_volumetric_extrusion_rate_slope\": \"0\"",
        "\"max_volumetric_extrusion_rate_slope\": \"300\"",
    );
    let output = crate::slice_project(&archive.bytes(), metadata())
        .await
        .unwrap();
    let text = std::str::from_utf8(&output).unwrap();
    let last_layer = text.find("; Z_HEIGHT: 92\n").unwrap();
    let end = text.find("; EXECUTABLE_BLOCK_END").unwrap();
    assert!(
        text[last_layer..end]
            .lines()
            .any(|line| line.starts_with("G1 X") && line.contains(" E"))
    );
}
