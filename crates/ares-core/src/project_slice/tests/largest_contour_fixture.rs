use crate::slice_project;

use super::support::{KsrArchive, metadata};

const PROJECT_SETTINGS: &str = "Metadata/project_settings.config";
const SPIRAL_OFF: &str = r#""spiral_mode": "0""#;
const SPIRAL_ON: &str = r#""spiral_mode": "1""#;
const BOTTOM_LAYERS_THREE: &str = r#""bottom_shell_layers": "3""#;
const BOTTOM_LAYERS_ZERO: &str = r#""bottom_shell_layers": "0""#;
const BOTTOM_THICKNESS_ZERO: &str = r#""bottom_shell_thickness": "0""#;
const BOTTOM_THICKNESS_VECTOR: &str = r#""bottom_shell_thickness": "0.5001""#;

#[tokio::test]
async fn task22h_public_global_spiral_reaches_largest_contour_and_gcode() {
    let output = slice_project(primary_mutation(), metadata()).await.unwrap();
    assert!(String::from_utf8_lossy(&output).contains(";LAYER_CHANGE"));
}

fn primary_mutation() -> Vec<u8> {
    let mut archive = KsrArchive::new();
    archive.replace_unique(PROJECT_SETTINGS, SPIRAL_OFF, SPIRAL_ON);
    archive.replace_unique(PROJECT_SETTINGS, BOTTOM_LAYERS_THREE, BOTTOM_LAYERS_ZERO);
    archive.replace_unique(
        PROJECT_SETTINGS,
        BOTTOM_THICKNESS_ZERO,
        BOTTOM_THICKNESS_VECTOR,
    );
    archive.bytes()
}
