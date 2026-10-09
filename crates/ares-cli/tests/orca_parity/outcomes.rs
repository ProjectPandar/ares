use super::ParityOutcome;

pub(crate) fn pass(label: &str) -> ParityOutcome {
    ParityOutcome {
        label: label.to_owned(),
        status: "PASS",
        detail: String::new(),
        artifacts: None,
    }
}

pub(crate) fn divergence(label: &str, difference: String) -> ParityOutcome {
    ParityOutcome {
        label: label.to_owned(),
        status: "DIVERGENT",
        detail: difference,
        artifacts: None,
    }
}
