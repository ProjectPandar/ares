#[path = "../../../tests/support/slice_project_plate_processed.rs"]
mod direct_project;
use ares_core::{ConfigOverrides, GenerationMetadata, SlicerContext};
use ares_wasm::SliceOutput;
use direct_project::slice_project_plate_processed;

// Failure inventory: a total or time lost, swapped, or rounded on the way to
// JSON (f32 times printed by their shortest f32 form). Values come from the
// core result, which the Native oracle checks in print_estimated_stats.
#[tokio::test]
async fn estimation_json_carries_exact_footer_inputs() {
    let processed = slice_project_plate_processed(
        include_bytes!("../../../tests/parity/summary/time-cost.3mf").to_vec(),
        GenerationMetadata::default(),
        Some(1),
        &ConfigOverrides::default(),
        SlicerContext::new(0),
    )
    .await
    .unwrap();
    let totals = processed.print_statistics;
    let times = &processed.result.print_statistics;
    let expected = [
        ("totalUsedFilament", totals.total_used_filament),
        ("totalWeight", totals.total_weight),
        ("totalCost", totals.total_cost),
        ("time", f64::from(times.normal_time)),
        ("prepareTime", f64::from(times.normal_prepare_time)),
    ];
    let output = SliceOutput::from_processed(processed.clone());
    // Read with float_roundtrip (dev-dependency): the default parser rounds
    // the 17-digit significand to f64 before scaling, a second rounding.
    let parsed: serde_json::Value = serde_json::from_str(&output.estimation()).unwrap();
    assert_eq!(parsed.as_object().unwrap().len(), expected.len());
    for (field, value) in expected {
        let actual = parsed[field].as_f64().unwrap();
        assert_eq!(actual.to_bits(), value.to_bits(), "{field}");
    }
}
