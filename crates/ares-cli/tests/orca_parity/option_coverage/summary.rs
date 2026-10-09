use super::{OptionOutcome, domains, parity, runner};

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

pub(super) fn write_summary(outcomes: &[OptionOutcome]) {
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
        "# OrcaSlicer option coverage summary\n\n{pass}/{} declared-endpoint/interior scopes complete; {generated} generated, {compared} strictly compared, {rejected} rejected. Not all-printer/default coverage.\n\n| status | option | type | cases | compared | rejected | upstream | first result |\n|---|---|---|---:|---:|---:|---|---|\n",
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
    let boundaries = domains::numeric_boundaries::inventory(&runner::repo_root());
    let unbounded = boundaries
        .iter()
        .flat_map(|row| {
            ["minimum", "maximum"]
                .into_iter()
                .filter(move |side| row[*side]["kind"] == "UNBOUNDED")
                .map(move |side| {
                    serde_json::json!({
                        "key": row["key"], "side": side, "source": row["source"]
                    })
                })
        })
        .collect::<Vec<_>>();
    parity::artifacts::overwrite(
        &root.join("numeric-boundaries.json"),
        &serde_json::to_vec_pretty(&serde_json::json!({
            "scope": "declared endpoints plus existing interior samples; not full legal-range coverage",
            "unbounded_default_source": "src/libslic3r/Config.hpp:2475-2479",
            "unbounded_side_count": unbounded.len(), "unbounded_sides": unbounded,
            "options": boundaries
        })).unwrap(),
    ).unwrap();
    let inventory = plans.iter().map(|plan| {
        let mut row = serde_json::json!({
            "key": plan.key, "type": plan.option_type, "source": plan.source,
            "declared_scope_complete": plan.provably_legal, "omission": plan.omission,
            "cases": plan.cases.iter().map(|case| serde_json::json!({"label":case.label,"value":case.value})).collect::<Vec<_>>()
        });
        if plan.key == super::adaptive_pa_bridges::KEY {
            row["bounded_domain"] = super::adaptive_pa_bridges::metadata();
        } else if plan.key == super::print_compatible_printers::KEY {
            row["bounded_domain"] = super::print_compatible_printers::metadata();
        }
        row
    }).collect::<Vec<_>>();
    parity::artifacts::overwrite(
        &root.join("option-domain-plans.json"),
        &serde_json::to_vec_pretty(&inventory).unwrap(),
    )
    .unwrap();
}
