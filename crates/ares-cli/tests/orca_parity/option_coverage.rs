//! Option-domain parity sweep against the Ender-3 smoke fixture.

#[path = "option_coverage/activation.rs"]
mod activation;
use activation::{inject_case, support_geometry_option};

#[path = "option_coverage/domains.rs"]
mod domains;
#[cfg(test)]
#[path = "option_coverage/float_percent_tests.rs"]
mod float_percent_tests;
#[cfg(test)]
#[path = "option_coverage/owner_pair_tests.rs"]
mod owner_pair_tests;
#[cfg(test)]
#[path = "option_coverage/sweep.rs"]
mod sweep;
#[cfg(test)]
#[path = "option_coverage/tests.rs"]
mod tests;

#[cfg(test)]
#[path = "option_coverage/completion_tests.rs"]
mod completion_tests;

#[cfg(test)]
#[path = "option_coverage/pressure_advance.rs"]
mod pressure_advance;

#[cfg(test)]
#[path = "option_coverage/skirt_height.rs"]
mod skirt_height;

#[cfg(test)]
#[path = "option_coverage/hot_plate.rs"]
mod hot_plate;

#[cfg(test)]
#[path = "option_coverage/fill_grid.rs"]
mod fill_grid;

#[cfg(test)]
#[path = "option_coverage/monotonic_chain.rs"]
mod monotonic_chain;

#[cfg(test)]
#[path = "option_coverage/locked_widths.rs"]
mod locked_widths;

#[cfg(test)]
#[path = "option_coverage/cut_retraction.rs"]
mod cut_retraction;

#[cfg(test)]
#[path = "option_coverage/cut_retraction_values.rs"]
mod cut_retraction_values;

#[cfg(test)]
#[path = "option_coverage/air_filtration.rs"]
mod air_filtration;

#[cfg(test)]
#[path = "option_coverage/chamber_control.rs"]
mod chamber_control;

#[cfg(test)]
#[path = "option_coverage/scalar_points.rs"]
mod scalar_points;

#[cfg(test)]
#[path = "option_coverage/obsolete_silent_mode.rs"]
mod obsolete_silent_mode;

#[cfg(test)]
#[path = "option_coverage/resolution_floor.rs"]
mod resolution_floor;

#[cfg(test)]
#[path = "option_coverage/filament_map_mode.rs"]
mod filament_map_mode;

#[cfg(test)]
#[path = "option_coverage/thumbnails.rs"]
mod thumbnails;

#[cfg(test)]
#[path = "option_coverage/preset_identity_tests.rs"]
mod preset_identity_tests;

#[path = "option_coverage/preset_identity.rs"]
mod preset_identity;

use crate::{
    self as parity,
    presets::VendorProfiles,
    runner::{self, OrcaRunner},
};
use serde_json::{Map, Value};
use std::path::PathBuf;

fn profiles_root() -> PathBuf {
    runner::repo_root().join("OrcaSlicer/resources/profiles")
}

fn strict_comparator_ran(status: &str) -> bool {
    matches!(status, "PASS" | "DIVERGENT")
}

struct OptionOutcome {
    key: String,
    option_type: String,
    source: String,
    cases: usize,
    compared: usize,
    rejected: usize,
    status: &'static str,
    detail: String,
}

impl OptionOutcome {
    fn executed(
        plan: &domains::OptionPlan,
        compared: usize,
        rejected: Vec<String>,
        excused: usize,
        unimplemented: usize,
        failure: Option<String>,
    ) -> Self {
        let (status, detail) = if let Some(detail) = failure {
            ("FAIL", detail)
        } else if unimplemented > 0 && compared + excused + unimplemented == plan.cases.len() {
            // Every non-excused case lands in the named
            // unimplemented-support-material boundary — a missing
            // upstream port, distinct from nondeterministic-oracle
            // excuses and from genuine byte regressions.
            (
                "INCOMPLETE",
                format!(
                    "{unimplemented} case(s) diverge: upstream feature unimplemented in ares \
                     (support-material port pending; PrintObjectSupportMaterial/TreeSupport)"
                ),
            )
        } else if compared == 0
            || compared + excused + unimplemented != plan.cases.len()
            || !rejected.is_empty()
            || plan.cases.iter().any(|case| case.value.is_none())
        {
            (
                "INCOMPLETE",
                format!(
                    "required legal cases not all compared; {}",
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
            // Baseline-only/unbounded domains carry no legality proof: a
            // single default case cannot establish that the whole legal range
            // was exercised. Enumerable (bool/enum), schema-bounded, and
            // context-bounded width domains are provably legal by construction.
            // An authored-probe omission marks a deliberately-sampled domain
            // (schema bounds rejected by upstream validation) — still
            // INCOMPLETE, so it does not pass the sweep completion gate.
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

    fn omitted(plan: &domains::OptionPlan) -> Self {
        Self::new(
            plan,
            (0, 0, 0),
            "UNBOUNDED",
            plan.omission.unwrap_or_default().to_owned(),
        )
    }

    fn new(
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

struct PriorRow {
    status: String,
    cases: usize,
    compared: usize,
    rejected: usize,
    raw: String,
}

/// Parse a previously-written `option-coverage-summary.md` back into
/// per-key rows so a filtered `ARES_OPTION_KEYS` sweep can merge its fresh
/// outcomes over the prior full sweep instead of clobbering them.
fn read_summary_rows(path: &std::path::Path) -> std::collections::BTreeMap<String, PriorRow> {
    let Ok(text) = std::fs::read_to_string(path) else {
        return Default::default();
    };
    text.lines()
        .filter(|line| line.starts_with("| "))
        .filter_map(|line| {
            let cells: Vec<&str> = line.trim_matches('|').split('|').map(str::trim).collect();
            // Skip the header and separator rows.
            if cells.len() != 8 || cells[0] == "status" || cells[0].starts_with('-') {
                return None;
            }
            let mut raw = line.to_string();
            raw.push('\n');
            Some((
                cells[1].to_string(),
                PriorRow {
                    status: cells[0].to_string(),
                    cases: cells[3].parse().unwrap_or(0),
                    compared: cells[4].parse().unwrap_or(0),
                    rejected: cells[5].parse().unwrap_or(0),
                    raw,
                },
            ))
        })
        .collect()
}

fn write_summary(outcomes: &[OptionOutcome]) {
    let root = parity::artifacts::root_from_env().unwrap();
    let summary_path = root.join("option-coverage-summary.md");
    // Merge with any prior run: a targeted `ARES_OPTION_KEYS` sweep only
    // produces rows for the filtered keys, so rows for untouched keys are
    // carried forward from the existing summary rather than dropped.
    let fresh: std::collections::BTreeMap<&str, &OptionOutcome> =
        outcomes.iter().map(|o| (o.key.as_str(), o)).collect();
    let prior = read_summary_rows(&summary_path);
    // Emit rows in canonical inventory order so the summary is stable across
    // full and filtered runs.
    let canonical = domains::load(&runner::repo_root());
    let mut merged: Vec<String> = Vec::new();
    let mut pass = 0usize;
    let mut generated = 0usize;
    let mut compared = 0usize;
    let mut rejected = 0usize;
    for plan in &canonical {
        if let Some(outcome) = fresh.get(plan.key.as_str()) {
            let row = format!(
                "| {} | {} | {} | {} | {} | {} | {} | {} |\n",
                outcome.status,
                outcome.key,
                outcome.option_type,
                outcome.cases,
                outcome.compared,
                outcome.rejected,
                outcome.source,
                outcome.detail.replace(['\n', '|'], " ")
            );
            if outcome.status == "PASS" {
                pass += 1;
            }
            generated += outcome.cases;
            compared += outcome.compared;
            rejected += outcome.rejected;
            merged.push(row);
        } else if let Some(row) = prior.get(&plan.key) {
            if row.status == "PASS" {
                pass += 1;
            }
            generated += row.cases;
            compared += row.compared;
            rejected += row.rejected;
            merged.push(row.raw.clone());
        }
    }
    let mut output = format!(
        "# OrcaSlicer option coverage summary\n\n{pass}/{} bounded domains complete; {generated} generated, {compared} strictly compared, {rejected} rejected. Not all-printer/default coverage.\n\n| status | option | type | cases | compared | rejected | upstream | first result |\n|---|---|---|---:|---:|---:|---|---|\n",
        merged.len()
    );
    for row in &merged {
        output.push_str(row);
    }
    parity::artifacts::overwrite(&summary_path, output.as_bytes()).unwrap();
    let plans = domains::load(&runner::repo_root());
    let widths = plans.iter().filter_map(|plan| plan.width_domain.as_ref().map(|domain| {
        serde_json::json!({"key": plan.key, "domain": domain.metadata, "nonexecuted_probes": domain.probes})
    })).collect::<Vec<_>>();
    parity::artifacts::overwrite(
        &root.join("width-domains.json"),
        &serde_json::to_vec_pretty(&widths).unwrap(),
    )
    .unwrap();
}
