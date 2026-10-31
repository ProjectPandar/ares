use std::{collections::BTreeMap, process::Command};

use serde_json::Value;

use super::OrcaRunner;

#[test]
fn support_report_counts_every_compared_type_and_preserves_divergence() {
    if OrcaRunner::from_env().unwrap().is_none() {
        return;
    }
    let root = tempfile::Builder::new()
        .prefix("support-report-")
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    eprintln!("support report artifacts: {root:?}");
    let output = Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "option_coverage::sweep::orca_parity_option_coverage",
            "--nocapture",
        ])
        .env("ARES_PARITY_OPTIONS", "1")
        .env("ARES_OPTION_KEYS", "support_type")
        .env_remove("ARES_OPTION_VALUES")
        .env_remove("ARES_PARITY_PATTERN")
        .env("ARES_PARITY_ARTIFACT_ROOT", &root)
        .output()
        .unwrap();
    let summary = std::fs::read_to_string(root.join("option-coverage-summary.md")).unwrap();
    let row = summary
        .lines()
        .find(|line| line.contains("| support_type |"))
        .unwrap();
    let columns = row
        .trim_matches('|')
        .split('|')
        .map(str::trim)
        .collect::<Vec<_>>();
    let mut statuses = BTreeMap::new();
    for entry in std::fs::read_dir(&root).unwrap().map(Result::unwrap) {
        if !entry.file_name().to_str().unwrap().starts_with("case-") {
            continue;
        }
        let manifest: Value =
            serde_json::from_slice(&std::fs::read(entry.path().join("manifest.json")).unwrap())
                .unwrap();
        statuses.insert(
            manifest["label"].as_str().unwrap().to_owned(),
            manifest["status"].as_str().unwrap().to_owned(),
        );
    }
    assert_eq!(
        statuses.keys().map(String::as_str).collect::<Vec<_>>(),
        [
            "option/support_type/normal(auto)",
            "option/support_type/normal(manual)",
            "option/support_type/tree(auto)",
            "option/support_type/tree(manual)",
        ]
    );
    assert!(
        statuses
            .values()
            .all(|status| matches!(status.as_str(), "PASS" | "DIVERGENT"))
    );
    assert_eq!(columns[3], "4", "{row}");
    assert_eq!(
        columns[4], "4",
        "all complete comparisons must count: {row}"
    );
    let divergent = statuses.values().any(|status| status == "DIVERGENT");
    assert_eq!(columns[0], if divergent { "FAIL" } else { "PASS" }, "{row}");
    assert_eq!(output.status.success(), !divergent, "{summary}");
    if divergent {
        assert!(columns[7].contains("first difference"), "{row}");
    }
    assert!(!summary.contains("support-material port pending"));
}
