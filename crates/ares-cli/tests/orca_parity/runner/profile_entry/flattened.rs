use ares_core::{
    ConfigOverrides, LoadSettingsFile, LoadSettingsSliceRequest, Placement, SliceError,
};
use std::{fs, path::PathBuf};

pub(in crate::runner) fn slice_flattened(
    (input, filename): (&[u8], &str),
    settings_paths: &[PathBuf],
    filament_paths: &[PathBuf],
    overrides: &ConfigOverrides,
    placement: Placement,
) -> Result<Vec<u8>, SliceError> {
    let settings_text: Vec<_> = settings_paths
        .iter()
        .map(|path| fs::read_to_string(path).unwrap())
        .collect();
    let filament_text: Vec<_> = filament_paths
        .iter()
        .map(|path| fs::read_to_string(path).unwrap())
        .collect();
    let settings: Vec<_> = settings_paths
        .iter()
        .zip(&settings_text)
        .map(|(path, json)| LoadSettingsFile {
            label: path.to_str().unwrap(),
            json,
            source_identity: None,
        })
        .collect();
    let filaments: Vec<_> = filament_paths
        .iter()
        .zip(&filament_text)
        .map(|(path, json)| LoadSettingsFile {
            label: path.to_str().unwrap(),
            json,
            source_identity: None,
        })
        .collect();
    // OrcaSlicer.cpp:2121-2123 skips preset resolution for typed files without inherits.
    tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
        .block_on(ares_core::slice_with_load_settings(
            input,
            LoadSettingsSliceRequest {
                filename,
                catalog_json: None,
                loaded_sources: None,
                uptodate: false,
                settings: &settings,
                filaments: &filaments,
                plate: None,
                overrides,
                placement,
                sliced_plate: &mut ares_core::SlicedPlate::default(),
            },
            ares_core::GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
            ares_core::SlicerContext::new(0),
        ))
}
