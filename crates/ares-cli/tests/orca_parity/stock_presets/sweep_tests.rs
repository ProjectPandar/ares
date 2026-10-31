use super::{ParityOutcome, finish};

fn profiles_root() -> std::path::PathBuf {
    crate::runner::repo_root().join("OrcaSlicer/resources/profiles")
}

#[test]
fn official_mode_marks_random_start_gcode_printers_skip() {
    let profiles = crate::presets::VendorProfiles::load(&profiles_root(), "Flashforge").unwrap();
    let random = profiles
        .machine("Flashforge Guider 3 Ultra 0.4 Nozzle")
        .unwrap();
    assert_eq!(
        super::random_start_gcode_skip(&random, false),
        Some("seeded-only: random() in machine_start_gcode, PlaceholderParser.cpp:2319")
    );
    // Seeded mode pins the RNG (ORCA_REPLAY_SEED=0); those printers stay on
    // the strict whole-file comparison path.
    assert_eq!(super::random_start_gcode_skip(&random, true), None);
    let plain = profiles.machine("Flashforge Guider4 0.25 nozzle").unwrap();
    assert!(plain.get("machine_start_gcode").is_some());
    assert_eq!(super::random_start_gcode_skip(&plain, false), None);
    // A machine without the key must not skip.
    let bare = serde_json::Map::new();
    assert_eq!(super::random_start_gcode_skip(&bare, false), None);
}

#[test]
fn stock_printer_sweep() {
    super::run();
}

fn row(status: &'static str) -> ParityOutcome {
    ParityOutcome {
        label: "vendor/printer".into(),
        status,
        detail: "native diagnostic".into(),
        artifacts: None,
    }
}

#[test]
fn stock_sweep_persists_every_error_before_rejecting_incomplete_coverage() {
    let root = tempfile::tempdir().unwrap();
    let result = finish(root.path(), &[row("PASS"), row("ORCA_ERROR")]);
    let report: serde_json::Value = serde_json::from_slice(
        &std::fs::read(root.path().join("stock-printer-summary.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(report["outcome_rows"], 2);
    assert_eq!(report["passed_rows"], 1);
    assert_eq!(report["complete"], false);
    assert_eq!(report["rows"][1]["status"], "ORCA_ERROR");
    assert_eq!(report["rows"][1]["detail"], "native diagnostic");
    assert!(result.is_err());
}

#[test]
fn stock_sweep_requires_a_nonempty_all_pass_inventory() {
    let root = tempfile::tempdir().unwrap();
    assert!(finish(root.path(), &[row("PASS")]).is_ok());
    let report: serde_json::Value = serde_json::from_slice(
        &std::fs::read(root.path().join("stock-printer-summary.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(report["complete"], true);
    assert!(finish(root.path(), &[]).is_err());
    for status in [
        "DIVERGENT",
        "ARES_ERROR",
        "VENDOR_INCOMPLETE",
        "INPUT_REJECTED",
        "ARTIFACT_ERROR",
        "UNKNOWN",
    ] {
        assert!(finish(root.path(), &[row("PASS"), row(status)]).is_err());
    }
}
