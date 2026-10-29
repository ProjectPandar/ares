//! `OptionOutcome` bookkeeping for option-domain coverage legs — moved out
//! of `option_coverage.rs` to keep the index file under the 400-LOC cap.
//! Re-exported at the module root so `super::{OptionOutcome, …}` paths in
//! existing coverage legs stay unchanged.

use super::domains;
use crate::runner;
use std::path::PathBuf;

pub(super) fn profiles_root() -> PathBuf {
    runner::repo_root().join("OrcaSlicer/resources/profiles")
}

pub(super) fn strict_comparator_ran(status: &str) -> bool {
    matches!(status, "PASS" | "DIVERGENT")
}

pub(super) struct OptionOutcome {
    pub(super) key: String,
    pub(super) option_type: String,
    pub(super) source: String,
    pub(super) cases: usize,
    pub(super) compared: usize,
    pub(super) rejected: usize,
    pub(super) status: &'static str,
    pub(super) detail: String,
}

impl OptionOutcome {
    pub(super) fn executed(
        plan: &domains::OptionPlan,
        compared: usize,
        rejected: Vec<String>,
        excused: usize,
        failure: Option<String>,
    ) -> Self {
        let (status, detail) = if let Some(detail) = failure {
            ("FAIL", detail)
        } else if compared == 0
            || compared + excused != plan.cases.len()
            || !rejected.is_empty()
            || plan.cases.iter().any(|case| case.value.is_none())
        {
            let oracle_reason = if compared == 0 && excused > 0 {
                "ORACLE_NONDETERMINISTIC; "
            } else {
                ""
            };
            (
                "INCOMPLETE",
                format!(
                    "required legal cases not all compared; {oracle_reason}{}",
                    rejected.join("; ")
                ),
            )
        } else if excused > 0 {
            // Nondeterministic-oracle cases never byte-match by
            // construction; crediting them as strict coverage would
            // certify the domain on fewer verified values than it
            // declares. Report them as an evidenced incomplete instead.
            (
                "INCOMPLETE",
                format!("{excused} case(s) excused: oracle is nondeterministic for these values"),
            )
        } else if !plan.provably_legal {
            // Baseline-only, unresolved-expression and deferred context domains
            // cannot complete the declared-endpoint/interior scope.
            (
                "INCOMPLETE",
                plan.omission
                    .map(str::to_owned)
                    .unwrap_or_else(|| "legacy domain application/legality unverified".into()),
            )
        } else {
            ("PASS", String::new())
        };
        Self::new(
            plan,
            (plan.cases.len(), compared, rejected.len()),
            status,
            detail,
        )
    }

    pub(super) fn omitted(plan: &domains::OptionPlan) -> Self {
        Self::new(
            plan,
            (0, 0, 0),
            "UNBOUNDED",
            plan.omission.unwrap_or_default().to_owned(),
        )
    }

    pub(super) fn new(
        plan: &domains::OptionPlan,
        counts: (usize, usize, usize),
        status: &'static str,
        detail: String,
    ) -> Self {
        Self {
            key: plan.key.clone(),
            option_type: plan.option_type.clone(),
            source: plan.source.clone(),
            cases: counts.0,
            compared: counts.1,
            rejected: counts.2,
            status,
            detail,
        }
    }
}
