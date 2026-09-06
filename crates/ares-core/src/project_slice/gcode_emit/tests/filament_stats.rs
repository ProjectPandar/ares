use crate::project_slice::tests::support::{KsrArchive, metadata};

/// `GCode.cpp:2349-2368`: per-filament `[g]` and cost lines only appear
/// for extruders with positive weight/cost (the fixture is a BBL printer,
/// whose footer carries no total lines — verified against OrcaSlicer 2.4.2
/// output for printers with unset `filament_cost`).
#[tokio::test]
async fn unset_cost_omits_per_filament_cost_line() {
    const SETTINGS: &str = "Metadata/project_settings.config";
    let mut archive = KsrArchive::new();
    let mut settings: serde_json::Value =
        serde_json::from_str(&archive.entry_text(SETTINGS)).unwrap();
    settings["filament_cost"] = serde_json::json!(["0"]);
    archive.insert_text(SETTINGS, &serde_json::to_string(&settings).unwrap());

    let output = crate::slice_project(&archive.bytes(), metadata())
        .await
        .unwrap();
    let output = std::str::from_utf8(&output).unwrap();

    assert!(output.contains("\n; filament used [g] = "));
    assert!(!output.contains("\n; filament cost = "));
}
