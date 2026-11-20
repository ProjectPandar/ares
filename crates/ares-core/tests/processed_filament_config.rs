//! `GCodeProcessorResult::filament_diameters` / `filament_densities`
//! (`GCodeProcessor.hpp:252-254`): `GCodeProcessor::apply_config` sizes
//! both to `filament_count = filament_diameter.size()` (`GCodeProcessor.cpp:2033,
//! 2054-2056`) and fills slot `i` with `filament_diameter.get_at(i)` /
//! `filament_density.get_at(i)` as float (`:2081, 2083`); `get_at` falls back
//! to the first value (`Config.hpp:624-628`). A config without
//! `filament_density` reads its `PrintConfig` default `{ 0. }`
//! (`PrintConfig.cpp:2864-2870`). The expectations come from the raw
//! `project_settings.config` of a two-filament project.

use std::io::{Cursor, Read as _, Write as _};

use serde_json::{Map, Value};

const PROJECT: &[u8] = include_bytes!("../../../tests/ksr_fdmtest_v4/ksr_fdmtest_v4.project.3mf");
const SETTINGS: &str = "Metadata/project_settings.config";

/// `PROJECT` with its `project_settings.config` replaced by `edit(settings)`.
fn project_with(edit: fn(&mut Map<String, Value>)) -> (Vec<u8>, Map<String, Value>) {
    let mut archive = zip::ZipArchive::new(Cursor::new(PROJECT)).unwrap();
    let mut buffer = Cursor::new(Vec::new());
    let mut settings = Map::new();
    {
        let mut writer = zip::ZipWriter::new(&mut buffer);
        let options = zip::write::SimpleFileOptions::default();
        for index in 0..archive.len() {
            let mut file = archive.by_index(index).unwrap();
            let name = file.name().to_owned();
            let mut bytes = Vec::new();
            file.read_to_end(&mut bytes).unwrap();
            if name == SETTINGS {
                settings = serde_json::from_slice(&bytes).unwrap();
                edit(&mut settings);
                bytes = serde_json::to_vec(&settings).unwrap();
            }
            writer.start_file(name, options).unwrap();
            writer.write_all(&bytes).unwrap();
        }
        writer.finish().unwrap();
    }
    (buffer.into_inner(), settings)
}

fn floats(settings: &Map<String, Value>, key: &str) -> Vec<f32> {
    settings[key]
        .as_array()
        .unwrap()
        .iter()
        .map(|value| value.as_str().unwrap().parse::<f32>().unwrap())
        .collect()
}

async fn processed_after_system_preset_miss(project: Vec<u8>) -> ares_core::GCodeProcessorResult {
    let catalog = ares_core::PresetCatalogHandle::default();
    ares_core::slice_project_plate_processed_with_catalog(
        project,
        ares_core::GenerationMetadata::default(),
        (Some(1), &ares_core::ConfigOverrides::default()),
        Some(&catalog),
        ares_core::SlicerContext::new(0),
    )
    .await
    .unwrap()
    .result
}

#[tokio::test]
async fn filament_diameters_and_densities_follow_the_project_config() {
    let (project, settings) = project_with(|settings| {
        settings.insert(
            "filament_diameter".into(),
            serde_json::json!(["1.75", "2.85"]),
        );
        settings.insert(
            "filament_density".into(),
            serde_json::json!(["1.26", "1.31"]),
        );
    });
    let result = processed_after_system_preset_miss(project).await;
    assert_eq!(
        result.filament_diameters,
        floats(&settings, "filament_diameter")
    );
    assert_eq!(
        result.filament_densities,
        floats(&settings, "filament_density")
    );
}

#[tokio::test]
async fn missing_filament_density_reads_the_print_config_default() {
    let (project, settings) = project_with(|settings| {
        settings.remove("filament_density");
    });
    let result = processed_after_system_preset_miss(project).await;
    let diameters = floats(&settings, "filament_diameter");
    // `PrintConfig.cpp:2870`: `filament_density` defaults to `{ 0. }`, read through `get_at`.
    assert_eq!(result.filament_densities, vec![0.0; diameters.len()]);
    assert_eq!(result.filament_diameters, diameters);
}
