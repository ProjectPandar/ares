use super::super::support::KsrArchive;

const PROJECT_SETTINGS: &str = "Metadata/project_settings.config";
const RESOLUTION: &str = r#""resolution": "0.012""#;
const SPIRAL_OFF: &str = r#""spiral_mode": "0""#;
const SPIRAL_ON: &str = r#""spiral_mode": "1""#;
const BOTTOM_LAYERS_THREE: &str = r#""bottom_shell_layers": "3""#;
const BOTTOM_LAYERS_ZERO: &str = r#""bottom_shell_layers": "0""#;
const BOTTOM_LAYERS_TWENTY_ONE: &str = r#""bottom_shell_layers": "21""#;
const BOTTOM_THICKNESS_ZERO: &str = r#""bottom_shell_thickness": "0""#;
const BOTTOM_THICKNESS_VECTOR: &str = r#""bottom_shell_thickness": "0.5001""#;

pub(super) fn resolution(value: &str) -> Vec<u8> {
    let mut archive = KsrArchive::new();
    archive.replace_unique(
        PROJECT_SETTINGS,
        RESOLUTION,
        &format!(r#""resolution": "{value}""#),
    );
    archive.bytes()
}

pub(super) fn primary() -> Vec<u8> {
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

pub(super) fn threshold_21() -> Vec<u8> {
    let mut archive = KsrArchive::new();
    archive.replace_unique(PROJECT_SETTINGS, SPIRAL_OFF, SPIRAL_ON);
    archive.replace_unique(
        PROJECT_SETTINGS,
        BOTTOM_LAYERS_THREE,
        BOTTOM_LAYERS_TWENTY_ONE,
    );
    archive.bytes()
}
