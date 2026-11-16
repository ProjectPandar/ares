use ares_core::{
    ConfigOverrides, GenerationMetadata, LoadSettingsFile, LoadSettingsSliceRequest,
    LoadedPresetSources, Placement, PresetAdmissionReason, SliceError, SlicedPlate, SlicerContext,
    SourceIdentity,
};

const PROJECT: &[u8] =
    include_bytes!("../../../tests/parity/validation/load-settings-3mf/compatible-input.3mf");
const MACHINE: &str =
    include_str!("../../../tests/parity/validation/load-settings-3mf/machine.json");
const PROCESS: &str =
    include_str!("../../../tests/parity/validation/load-settings-3mf/process.json");
const FILAMENT: &str =
    include_str!("../../../tests/parity/validation/load-settings-3mf/compatible-filament-0.json");

async fn run(
    sources: &LoadedPresetSources,
    identity: &SourceIdentity,
    inherited: bool,
) -> Result<Vec<u8>, SliceError> {
    let mut machine: serde_json::Value = serde_json::from_str(MACHINE).unwrap();
    if inherited {
        machine["from"] = "user".into();
        machine["inherits"] = "external parent must never be resolved by name".into();
        machine["name"] = "external name differs from canonical".into();
    }
    let json = machine.to_string();
    ares_core::slice_with_load_settings(
        PROJECT,
        LoadSettingsSliceRequest {
            filename: "compatible-input.3mf",
            catalog_json: None,
            loaded_sources: Some(sources),
            uptodate: false,
            settings: &[
                LoadSettingsFile {
                    label: "external.json",
                    json: &json,
                    source_identity: Some(identity),
                },
                LoadSettingsFile {
                    label: "process.json",
                    json: PROCESS,
                    source_identity: None,
                },
            ],
            filaments: &[LoadSettingsFile {
                label: "filament.json",
                json: FILAMENT,
                source_identity: None,
            }],
            plate: None,
            overrides: &ConfigOverrides::default(),
            placement: Placement::None,
            orient: false,
            sliced_plate: &mut SlicedPlate::default(),
        },
        GenerationMetadata::deterministic(2026, 11, 16, 0, 0, 0),
        SlicerContext::new(0),
    )
    .await
}

#[tokio::test]
async fn source_admission_rejects_unloaded_same_name_and_bytes() {
    let identity = SourceIdentity::from_host_bytes([1].as_slice());
    let mut sources = LoadedPresetSources::new();
    sources
        .add_loaded(MACHINE, &SourceIdentity::from_host_bytes([2].as_slice()))
        .unwrap();
    assert!(
        matches!(run(&sources, &identity, true).await, Err(SliceError::PresetAdmission(error)) if error.reason == PresetAdmissionReason::NotFound)
    );
}

#[tokio::test]
async fn source_admission_unique_identity_replaces_external_config() {
    let identity = SourceIdentity::from_host_bytes([1].as_slice());
    let mut sources = LoadedPresetSources::new();
    sources.add_loaded(MACHINE, &identity).unwrap();
    let output = run(&sources, &identity, true).await.unwrap();
    assert!(!output.is_empty());
    assert!(!String::from_utf8_lossy(&output).contains("external parent must never"));
}

#[tokio::test]
async fn source_admission_rejects_ambiguous_identity() {
    let identity = SourceIdentity::from_host_bytes([1].as_slice());
    let mut sources = LoadedPresetSources::new();
    sources.add_loaded(MACHINE, &identity).unwrap();
    sources.add_loaded(MACHINE, &identity).unwrap();
    assert!(
        matches!(run(&sources, &identity, true).await, Err(SliceError::PresetAdmission(error)) if error.reason == PresetAdmissionReason::Ambiguous)
    );
}

#[tokio::test]
async fn source_admission_standalone_bypasses_identity_lookup() {
    let identity = SourceIdentity::from_host_bytes([1].as_slice());
    assert!(
        !run(&LoadedPresetSources::new(), &identity, false)
            .await
            .unwrap()
            .is_empty()
    );
}
