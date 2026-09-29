//! Option-domain parity sweep against the Ender-3 smoke fixture.

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

fn inject_case(
    plan: &domains::OptionPlan,
    case: &domains::OptionCase,
    machine: &mut Map<String, Value>,
    process: &mut Map<String, Value>,
    filaments: &mut [Map<String, Value>],
) {
    // Value-compatibility co-injections that must edit BOTH presets
    // run before the scope match borrows `machine`/`process`.
    normalize_case_dependencies(plan, case, machine, process, filaments);
    // Options that only take effect when another option is enabled must
    // co-inject that enabler, otherwise the case exercises a feature the
    // slicer never activates (e.g. `ironing_pattern` under `ironing_type =
    // "no ironing"`). (option -> (enabler, activating value))
    for (enabler, value) in activating_overrides(&plan.key) {
        // Enabler scope: `input_shaping_emit` is printer-scoped
        // (machine_envelope_config); every other enabler is process-scoped.
        let enabler_target = if *enabler == "input_shaping_emit" {
            &mut *machine
        } else {
            &mut *process
        };
        enabler_target.insert((*enabler).to_owned(), Value::String((*value).to_owned()));
    }
    let target = match plan.raw_scope.as_str() {
        "printer" => machine,
        "process" => process,
        "filament" => &mut filaments[0],
        "residual" if residual_is_machine(&plan.key) => machine,
        "residual" => process,
        scope => panic!("unknown option scope {scope}"),
    };
    if let Some(value) = &case.value {
        target.insert(plan.key.clone(), value.clone());
    }
    // Upstream validation rejects some option VALUES in combination with
    // an incompatible baseline sibling — the value itself is legal, the
    // fixture is not. Co-inject the dependent setting so the oracle
    // actually slices the probed value (`Print.cpp:1740-1746` G92 guard,
    // `--use-firmware-retraction is not compatible with --wipe`).
}

fn normalize_case_dependencies(
    plan: &domains::OptionPlan,
    case: &domains::OptionCase,
    machine: &mut Map<String, Value>,
    process: &mut Map<String, Value>,
    filaments: &mut [Map<String, Value>],
) {
    let active = case
        .value
        .as_ref()
        .and_then(Value::as_str)
        .unwrap_or_default();
    if plan.key == "use_firmware_retraction" && active == "1" {
        // `wipe` is a filament-scoped option — `--use-firmware-retraction`
        // is rejected while wiping stays enabled on the material preset.
        for filament in filaments.iter_mut() {
            filament.insert("wipe".to_owned(), Value::String("0".to_owned()));
        }
    }
    if plan.key == "use_relative_e_distances" && active == "0" {
        for key in ["before_layer_change_gcode", "layer_change_gcode"] {
            for preset in [&mut *machine, &mut *process] {
                if let Some(text) = preset.get(key).and_then(Value::as_str).map(str::to_owned)
                    && text.contains("G92 E0")
                {
                    // Replace the absolute-mode-incompatible reset with a
                    // neutral comment, keeping any other lines.
                    let normalized = text
                        .lines()
                        .map(|line| {
                            if line.trim_start().starts_with("G92 E0") {
                                "; absolute-mode case: reset removed"
                            } else {
                                line
                            }
                        })
                        .collect::<Vec<_>>()
                        .join("\n");
                    preset.insert(key.to_owned(), Value::String(normalized));
                }
            }
        }
    }
}

/// Enabler (key, activating value) pairs for options gated behind a disabled
/// feature in the baseline process preset.
/// Support-structure options need an overhanging model — on a solid cube
/// `enable_support = 1` produces no support layers and the whole family
/// stays inert. `tests/parity/overhang.stl` (T-shape, 10mm cantilevers)
/// produces ordinary + interface support geometry under the same preset.
/// Raft options stay on the cube: `raft_layers > 0` alone generates the
/// raft regardless of overhang geometry (`Print.hpp:430`).
pub(crate) fn support_geometry_option(key: &str) -> bool {
    key.starts_with("support_") || key.starts_with("tree_support_")
}

fn activating_overrides(key: &str) -> &'static [(&'static str, &'static str)] {
    match key {
        // Ironing sub-options only run when `ironing_type != "no ironing"`.
        "ironing_flow" | "ironing_spacing" | "ironing_speed" | "ironing_pattern" => {
            &[("ironing_type", "top")]
        }
        // Fuzzy-skin geometry params only run when `fuzzy_skin != "none"`.
        "fuzzy_skin_point_dist" | "fuzzy_skin_point_type" => &[("fuzzy_skin", "external")],
        "fuzzy_skin_first_layer"
        | "fuzzy_skin_layers_between_ripple_offset"
        | "fuzzy_skin_mode"
        | "fuzzy_skin_noise_type"
        | "fuzzy_skin_point_distance"
        | "fuzzy_skin_ripple_offset"
        | "fuzzy_skin_ripples_per_layer"
        | "fuzzy_skin_thickness" => &[("fuzzy_skin", "external")],
        // Coherent-noise params are consumed only by the non-ripple noise
        // modules (`FuzzySkin.cpp:296-301` bypasses them for ripple), so
        // the domain must activate a coherent noise type.
        "fuzzy_skin_octaves" | "fuzzy_skin_persistence" | "fuzzy_skin_scale" => &[
            ("fuzzy_skin", "external"),
            ("fuzzy_skin_noise_type", "perlin"),
        ],
        // Input-shaping params only emit `M593`/`SET_INPUT_SHAPER` when
        // `input_shaping_emit` is on (`GCode.cpp:3991`); the marlin
        // baseline is already `gcfMarlinFirmware`, so no flavor override.
        "input_shaping_damp_x"
        | "input_shaping_damp_y"
        | "input_shaping_freq_x"
        | "input_shaping_freq_y"
        | "input_shaping_type" => &[("input_shaping_emit", "1")],
        // Support structure sub-options only run when supports generate
        // (`support_layers`/`raft`/`tree_support_*` all gate on
        // `enable_support`). `support_type`/`support_style` select the
        // generator and stay active on the cube only when supports are
        // enabled AND the model overhangs — the sweep uses
        // `tests/parity/overhang.stl` for these keys (see
        // `support_geometry_option`).
        "support_angle"
        | "support_air_filtration"
        | "support_base_pattern"
        | "support_base_pattern_spacing"
        | "support_bottom_interface_spacing"
        | "support_bottom_z_distance"
        | "support_chamber_temp_control"
        | "support_critical_regions_only"
        | "support_expansion"
        | "support_filament"
        | "support_flow_ratio"
        | "support_interface_bottom_layers"
        | "support_interface_filament"
        | "support_interface_flow_ratio"
        | "support_interface_loop_pattern"
        | "support_interface_not_for_body"
        | "support_interface_pattern"
        | "support_interface_spacing"
        | "support_interface_speed"
        | "support_interface_top_layers"
        | "support_ironing"
        | "support_ironing_flow"
        | "support_ironing_pattern"
        | "support_ironing_spacing"
        | "support_line_width"
        | "support_material_angle"
        | "support_material_enforce_layers"
        | "support_material_extruder"
        | "support_material_interface_extruder"
        | "support_material_interface_fan_speed"
        | "support_multi_bed_types"
        | "support_object_first_layer_gap"
        | "support_object_skip_flush"
        | "support_object_xy_distance"
        | "support_on_build_plate_only"
        | "support_parallel_printheads"
        | "support_remove_small_overhang"
        | "support_speed"
        | "support_style"
        | "support_threshold_angle"
        | "support_threshold_overlap"
        | "support_top_z_distance"
        | "support_type" => &[("enable_support", "1")],
        // Raft params gate on `raft_layers > 0` (`Print.hpp:430`), not
        // `enable_support`; a raft does not need support enabled.
        "raft_contact_distance"
        | "raft_expansion"
        | "raft_first_layer_density"
        | "raft_first_layer_expansion" => &[("raft_layers", "2")],
        // Tree-support params only apply when the tree generator is
        // selected (`support_type` default is `normal(auto)`).
        "tree_support_angle_slow"
        | "tree_support_auto_brim"
        | "tree_support_branch_angle"
        | "tree_support_branch_angle_organic"
        | "tree_support_branch_diameter"
        | "tree_support_branch_diameter_angle"
        | "tree_support_branch_diameter_organic"
        | "tree_support_branch_distance"
        | "tree_support_branch_distance_organic"
        | "tree_support_brim_width"
        | "tree_support_tip_diameter"
        | "tree_support_top_rate"
        | "tree_support_wall_count" => &[("enable_support", "1"), ("support_type", "tree(auto)")],
        // Gap-fill params only run when gap filling is enabled.
        "gap_fill_target" => &[("gap_infill_type", "everywhere")],
        // Wrapping templates and their exclude area only emit when
        // detection is on (`GCode.cpp:5302-5305`).
        "wrapping_detection_gcode" | "wrapping_exclude_area" => {
            &[("enable_wrapping_detection", "1")]
        }
        _ => &[],
    }
}

fn residual_is_machine(key: &str) -> bool {
    matches!(
        key,
        "deretraction_speed"
            | "extruder_offset"
            | "max_layer_height"
            | "min_layer_height"
            | "nozzle_diameter"
            | "nozzle_volume_type"
            | "retract_before_wipe"
            | "retract_length_toolchange"
            | "retract_lift_above"
            | "retract_lift_below"
            | "retract_restart_extra"
            | "retract_restart_extra_toolchange"
            | "retraction_length"
            | "retraction_minimum_travel"
            | "retraction_speed"
            | "wipe"
            | "wipe_distance"
            | "z_hop"
    )
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
            // INCOMPLETE, but the sweep gate treats it as covered interior.
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
