use super::{parity::ParityOutcome, require_complete};

fn outcome(status: &'static str) -> ParityOutcome {
    ParityOutcome {
        label: "vendor/printer".into(),
        status,
        detail: "known diagnostic reason".into(),
        artifacts: None,
    }
}

#[test]
fn oracle_failure_does_not_complete_the_printer_sweep() {
    let rows = [outcome("PASS"), outcome("ORCA_ERROR")];
    assert!(std::panic::catch_unwind(|| require_complete(&rows)).is_err());
}

#[test]
fn missing_vendor_profile_does_not_complete_the_printer_sweep() {
    let rows = [outcome("PASS"), outcome("VENDOR_INCOMPLETE")];
    assert!(std::panic::catch_unwind(|| require_complete(&rows)).is_err());
}

#[test]
fn only_a_nonempty_all_pass_printer_sweep_completes() {
    require_complete(&[outcome("PASS"), outcome("PASS")]);
    assert!(std::panic::catch_unwind(|| require_complete(&[])).is_err());
    for status in [
        "DIVERGENT",
        "ARES_ERROR",
        "INPUT_REJECTED",
        "ARTIFACT_ERROR",
        "UNRECOGNIZED",
    ] {
        let rows = [outcome("PASS"), outcome(status)];
        assert!(std::panic::catch_unwind(|| require_complete(&rows)).is_err());
    }
}
