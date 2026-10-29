use ares_core::{
    ConfigOverrides, GenerationMetadata, SlicerContext, slice_project_plate_processed,
};
use ares_wasm::SliceOutput;

// Failure inventory: role map omitted, recomputed, converted to f32, or lost
// when the owned statistics buffer is taken. Core uses the Native reference.
#[tokio::test]
async fn wasm_statistics_transparently_preserve_native_role_usage() {
    let processed = slice_project_plate_processed(
        include_bytes!("../../../tests/parity/role-filament-usage/wasm-plate1.3mf").to_vec(),
        GenerationMetadata::default(),
        Some(1),
        &ConfigOverrides::default(),
        SlicerContext::new(0),
    )
    .await
    .unwrap();
    let expected = serde_json::to_vec(&processed.result.print_statistics).unwrap();
    let mut output = SliceOutput::from_processed(processed);
    let actual = output.take_print_statistics();
    assert_eq!(actual, expected);
    let parsed: serde_json::Value = serde_json::from_slice(&actual).unwrap();
    assert_eq!(
        parsed["used_filaments_per_role"].as_object().unwrap().len(),
        8
    );
    assert!(output.take_print_statistics().is_empty());
}
