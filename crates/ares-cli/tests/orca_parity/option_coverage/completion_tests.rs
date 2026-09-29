use super::{OptionOutcome, sweep::require_complete};

fn outcome(status: &'static str, compared: usize, detail: &str) -> OptionOutcome {
    OptionOutcome {
        key: "enable_support".into(),
        option_type: "coBool".into(),
        source: "src/libslic3r/PrintConfig.cpp".into(),
        cases: 2,
        compared,
        rejected: 0,
        status,
        detail: detail.into(),
    }
}

#[test]
fn incomplete_domains_fail_even_when_the_reason_is_known() {
    for (compared, detail) in [
        (0, "upstream feature unimplemented in ares"),
        (1, "upstream feature unimplemented in ares"),
        (1, "oracle is nondeterministic for these values"),
        (2, "authored probes sample the legal interior"),
        (2, "unbounded baseline only"),
    ] {
        assert!(
            require_complete(&[outcome("INCOMPLETE", compared, detail)]).is_err(),
            "incomplete coverage was accepted: {detail}"
        );
    }
}

#[test]
fn rejected_required_value_does_not_complete_a_domain() {
    let mut rejected = outcome("INCOMPLETE", 1, "required legal cases not all compared");
    rejected.rejected = 1;
    assert!(require_complete(&[rejected]).is_err());
}

#[test]
fn only_a_nonempty_complete_sweep_succeeds() {
    assert!(require_complete(&[]).is_err());
    assert!(require_complete(&[outcome("PASS", 2, "")]).is_ok());
    assert!(
        require_complete(&[
            outcome("PASS", 2, ""),
            outcome("FAIL", 2, "nondeterministic artifact path"),
        ])
        .is_err()
    );
    assert!(require_complete(&[outcome("UNBOUNDED", 0, "")]).is_err());
}
